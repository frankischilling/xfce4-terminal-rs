/*
 * Prints the link contract of the frozen C terminal widget.
 *
 * The frozen link helpers are file-private, so this probe includes the widget
 * implementation instead of linking against its object file. The remaining
 * frozen objects are linked unchanged, which keeps the printed contract a
 * report about the reference implementation rather than a reimplementation of
 * it.
 *
 * The probe replaces gtk_show_uri_on_window so the URI handed to the desktop
 * launcher can be recorded without starting a browser.
 */

#include <gtk/gtk.h>
#include <stdio.h>
#include <string.h>

#include "terminal-widget.c"


static gchar *launched_uri = NULL;


gboolean
gtk_show_uri_on_window (GtkWindow *parent,
                        const gchar *uri,
                        guint32 timestamp,
                        GError **error)
{
  (void) parent;
  (void) timestamp;
  (void) error;

  g_free (launched_uri);
  launched_uri = g_strdup (uri);
  return TRUE;
}


static const gchar *
pattern_type_name (PatternType type)
{
  switch (type)
    {
    case PATTERN_TYPE_NONE:
      return "none";
    case PATTERN_TYPE_FULL_HTTP:
      return "full-http";
    case PATTERN_TYPE_HTTP:
      return "http";
    case PATTERN_TYPE_EMAIL:
      return "email";
    case PATTERN_TYPE_FILE:
      return "file";
    default:
      g_error ("unknown pattern type %d", type);
    }
}


/*
 * Repeats the classification loop of terminal_widget_get_link for a string
 * that arrived as an OSC 8 hyperlink. The compiled patterns belong to the
 * frozen widget instance, so the pattern text, the compile options, and the
 * order of the table all come from the reference.
 */
static PatternType
classify (TerminalWidget *widget,
          const gchar *candidate)
{
  guint i;

  for (i = 0; i < G_N_ELEMENTS (regex_patterns); i++)
    {
      pcre2_match_data_8 *match_data;
      gint result;

      if (widget->regex_pcre[i] == NULL)
        continue;

      match_data = pcre2_match_data_create_from_pattern_8 (widget->regex_pcre[i], NULL);
      result = pcre2_match_8 (widget->regex_pcre[i], (PCRE2_SPTR8) candidate,
                              strlen (candidate), 0, 0, match_data, NULL);
      pcre2_match_data_free_8 (match_data);

      if (result >= 0)
        return regex_patterns[i].type;
    }

  return PATTERN_TYPE_NONE;
}


static GLogWriterOutput
print_log (GLogLevelFlags level,
           const GLogField *fields,
           gsize n_fields,
           gpointer data)
{
  gsize n;

  (void) data;

  for (n = 0; n < n_fields; n++)
    if (g_strcmp0 (fields[n].key, "MESSAGE") == 0)
      g_print ("log\t%s\t%s\n",
               (level & G_LOG_LEVEL_MASK) == G_LOG_LEVEL_WARNING ? "warning" : "other",
               (const gchar *) fields[n].value);

  return G_LOG_WRITER_HANDLED;
}


static void
report_candidate (TerminalWidget *widget,
                  const gchar *candidate)
{
  PatternType type = classify (widget, candidate);
  GtkWidget *item;
  gchar *copied;

  g_print ("classify\t%s\t%s\n", candidate, pattern_type_name (type));
  g_print ("clickable\t%s\t%s\n", candidate,
           terminal_widget_link_clickable (candidate, type) ? "true" : "false");

  g_clear_pointer (&launched_uri, g_free);
  terminal_widget_open_uri (widget, candidate, type);
  g_print ("launch\t%s\t%s\n", candidate,
           launched_uri != NULL ? launched_uri : "<none>");

  item = g_object_ref_sink (gtk_menu_item_new ());
  g_object_set_data_full (G_OBJECT (item), "terminal-widget-link",
                          g_strdup (candidate), g_free);
  terminal_widget_context_menu_copy (widget, item);
  copied = gtk_clipboard_wait_for_text (
    gtk_clipboard_get_for_display (gtk_widget_get_display (GTK_WIDGET (widget)),
                                   GDK_SELECTION_CLIPBOARD));
  g_print ("clipboard\t%s\t%s\n", candidate,
           copied != NULL ? copied : "<none>");
  g_free (copied);
  g_object_unref (item);
}


int
main (int argc,
      char **argv)
{
  TerminalWidget *widget;
  gchar *fixtures;
  gchar **candidates;
  guint i;

  gtk_init (&argc, &argv);

  if (argc != 2)
    {
      g_printerr ("usage: %s FIXTURE_FILE\n", argv[0]);
      return 2;
    }

  if (!g_file_get_contents (argv[1], &fixtures, NULL, NULL))
    {
      g_printerr ("cannot read %s\n", argv[1]);
      return 2;
    }

  widget = g_object_ref_sink (g_object_new (TERMINAL_TYPE_WIDGET, NULL));

  for (i = 0; i < G_N_ELEMENTS (regex_patterns); i++)
    g_print ("pattern\t%u\t%s\t%s\n", i,
             pattern_type_name (regex_patterns[i].type),
             regex_patterns[i].pattern);

  /* Report the messages of the measured calls instead of leaving them on the
   * standard error stream, where the isolated session adds unrelated noise. */
  g_log_set_writer_func (print_log, NULL, NULL);

  candidates = g_strsplit (fixtures, "\n", -1);
  for (i = 0; candidates[i] != NULL; i++)
    {
      /* The trailing newline of the file does not introduce a candidate. */
      if (candidates[i][0] == '#'
          || (candidates[i + 1] == NULL && candidates[i][0] == '\0'))
        continue;

      report_candidate (widget, candidates[i]);
    }

  g_strfreev (candidates);
  g_free (fixtures);
  return 0;
}
