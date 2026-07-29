/*
 * Writes the spawn request of the frozen C terminal screen.
 *
 * The command, the argument vector, and the child environment are decided by
 * file-private functions, so this probe includes the screen implementation
 * instead of linking against its object file. The remaining frozen objects are
 * linked unchanged, which keeps the written report a description of the
 * reference implementation rather than a second copy of it.
 *
 * Including the screen also brings its private fields into reach. The probe
 * writes the screen's own command there because a screen only receives one
 * through terminal_screen_new, which needs a tab attribute structure that the
 * option parser owns; the values the report contains still come from frozen
 * code.
 *
 * The report goes to a named file rather than to standard output, because the
 * display and session-bus wrappers the probe runs under add their own output.
 * It is written with stdio so that candidates keep their exact bytes; GLib's
 * printing functions would convert them to the current locale encoding.
 *
 * A realized toplevel contributes its window and display name to the child
 * environment. Those name resources of this run alone, so the probe records
 * them in a third file. The Rust candidate reads that file and reports the
 * environment of the same toplevel, because choosing and realizing a window
 * belongs to a widget layer the port has not reached yet.
 */

#include <gtk/gtk.h>
#include <stdio.h>
#include <string.h>

#include "terminal-screen.c"


static FILE *report = NULL;


/*
 * Repeats the argument vector that terminal_screen_launch_child hands to
 * vte_terminal_spawn_async. The command leads, the shell's own argument vector
 * follows, and the flags say that the first entry is the file to run.
 */
static void
report_spawn_request (guint index,
                      const gchar *command,
                      gchar **argv)
{
  GSpawnFlags flags = G_SPAWN_SEARCH_PATH;
  guint position = 0;
  guint i;

  fprintf (report, "argument\t%u\t%u\t%s\n", index, position++, command);
  if (argv != NULL)
    {
      for (i = 0; argv[i] != NULL; i++)
        fprintf (report, "argument\t%u\t%u\t%s\n", index, position++, argv[i]);
      flags |= G_SPAWN_FILE_AND_ARGV_ZERO;
    }
  fprintf (report, "spawn-flags\t%u\t%u\n", index, (guint) flags);
}


static void
report_command (TerminalScreen *screen,
                guint index,
                gchar **fields)
{
  GError *error = NULL;
  gchar *command = NULL;
  gchar **argv = NULL;

  g_object_set (G_OBJECT (screen->preferences),
                "command-login-shell", g_strcmp0 (fields[1], "true") == 0,
                "run-custom-command", g_strcmp0 (fields[2], "true") == 0,
                "custom-command", fields[3],
                NULL);

  /* A screen receives its own command from a tab attribute, which the option
   * parser fills in. The fixture supplies the same vector directly, and an
   * absent one leaves the terminating null of the split line. */
  terminal_screen_set_custom_command (screen, fields + 4);

  if (terminal_screen_get_child_command (screen, &command, &argv, &error))
    report_spawn_request (index, command, argv);
  else
    fprintf (report, "error\t%u\t%s\n", index, error->message);

  g_clear_error (&error);
  g_strfreev (argv);
  g_free (command);
}


static void
report_environment (TerminalScreen *screen,
                    guint index)
{
  gchar **environment = terminal_screen_get_child_environment (screen);
  guint i;

  for (i = 0; environment[i] != NULL; i++)
    fprintf (report, "variable\t%u\t%u\t%s\n", index, i, environment[i]);

  g_strfreev (environment);
}


/*
 * Records the window and display name of a realized toplevel so that the Rust
 * candidate can report the environment of the same one.
 */
static gboolean
write_toplevel (GtkWidget *toplevel,
                const gchar *path)
{
  GdkDisplay *display = gtk_widget_get_display (toplevel);
  FILE *file = fopen (path, "w");
  gboolean written = FALSE;

  if (file == NULL)
    return FALSE;

#ifdef ENABLE_X11
  if (GDK_IS_X11_DISPLAY (display))
    {
      fprintf (file, "x11\t%ld\t%s\n",
               (glong) gdk_x11_window_get_xid (gtk_widget_get_window (toplevel)),
               gdk_display_get_name (display));
      written = TRUE;
    }
#endif

  if (!written)
    fprintf (file, "other\t%s\n", gdk_display_get_name (display));

  return fclose (file) == 0 && written;
}


static void
report_toplevel (guint index,
                 const gchar *path)
{
  gchar *description;

  if (g_file_get_contents (path, &description, NULL, NULL))
    {
      g_strchomp (description);
      fprintf (report, "toplevel\t%u\t%s\n", index, description);
      g_free (description);
    }
}


int
main (int argc,
      char **argv)
{
  TerminalScreen *screen;
  TerminalScreen *windowed_screen;
  GtkWidget *window;
  gchar *fixtures;
  gchar **scenarios;
  guint index;

  gtk_init (&argc, &argv);

  if (argc != 4)
    {
      g_printerr ("usage: %s FIXTURE_FILE REPORT_FILE TOPLEVEL_FILE\n", argv[0]);
      return 2;
    }

  if (!g_file_get_contents (argv[1], &fixtures, NULL, NULL))
    {
      g_printerr ("cannot read %s\n", argv[1]);
      return 2;
    }

  report = fopen (argv[2], "w");
  if (report == NULL)
    {
      g_printerr ("cannot write %s\n", argv[2]);
      return 2;
    }

  /* The command scenarios read a screen that belongs to no window, which is
   * also the environment case without a realized toplevel. */
  screen = g_object_ref_sink (g_object_new (TERMINAL_TYPE_SCREEN, NULL));

  windowed_screen = g_object_new (TERMINAL_TYPE_SCREEN, NULL);
  window = g_object_ref_sink (gtk_window_new (GTK_WINDOW_TOPLEVEL));
  gtk_container_add (GTK_CONTAINER (window), GTK_WIDGET (windowed_screen));
  gtk_widget_realize (window);
  if (!write_toplevel (window, argv[3]))
    {
      g_printerr ("the realized toplevel is not an X11 window\n");
      return 3;
    }

  fprintf (report, "constant\tpty-flags\t%u\n", (guint) VTE_PTY_DEFAULT);
  fprintf (report, "constant\tspawn-timeout\t%d\n", SPAWN_TIMEOUT);

  scenarios = g_strsplit (fixtures, "\n", -1);
  for (index = 0; scenarios[index] != NULL; index++)
    {
      gchar **fields;

      /* The trailing newline of the file does not introduce a scenario. */
      if (scenarios[index][0] == '#'
          || (scenarios[index + 1] == NULL && scenarios[index][0] == '\0'))
        continue;

      fprintf (report, "scenario\t%u\t%s\n", index, scenarios[index]);
      fields = g_strsplit (scenarios[index], "\t", -1);

      if (g_strcmp0 (fields[0], "command") == 0
          && g_strv_length (fields) >= 4)
        {
          report_command (screen, index, fields);
        }
      else if (g_strcmp0 (fields[0], "environment") == 0
               && g_strcmp0 (fields[1], "plain") == 0)
        {
          report_environment (screen, index);
        }
      else if (g_strcmp0 (fields[0], "environment") == 0
               && g_strcmp0 (fields[1], "realized") == 0)
        {
          report_toplevel (index, argv[3]);
          report_environment (windowed_screen, index);
        }
      else
        {
          g_printerr ("unknown scenario on line %u\n", index);
          return 2;
        }

      g_strfreev (fields);
    }

  g_strfreev (scenarios);
  g_free (fixtures);
  g_object_unref (window);
  g_object_unref (screen);
  return fclose (report) == 0 ? 0 : 2;
}
