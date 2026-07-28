#include <glib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "terminal-options.h"


static void
print_string (const gchar *name,
              const gchar *value)
{
  if (value == NULL)
    g_print ("|%s=-", name);
  else
    {
      const gsize length = strlen (value);
      g_print ("|%s=%" G_GSIZE_FORMAT ":", name, length);
      fwrite (value, 1, length, stdout);
    }
}


static void
print_command (gchar **command)
{
  guint n;

  if (command == NULL)
    {
      g_print ("|command=-");
      return;
    }

  g_print ("|command=%u", g_strv_length (command));
  for (n = 0; command[n] != NULL; ++n)
    {
      const gsize length = strlen (command[n]);
      g_print (":%" G_GSIZE_FORMAT ":", length);
      fwrite (command[n], 1, length, stdout);
    }
}


int
main (int argc,
      char **argv)
{
  const gboolean can_reuse = g_getenv ("CAN_REUSE_WINDOW") != NULL;
  GSList *windows;
  GSList *wp;
  GError *error = NULL;

  windows = terminal_window_attr_parse (argc, argv, can_reuse, &error);
  if (windows == NULL)
    {
      fwrite (error->message, 1, strlen (error->message), stderr);
      fputc ('\n', stderr);
      g_error_free (error);
      return EXIT_FAILURE;
    }

  for (wp = windows; wp != NULL; wp = wp->next)
    {
      TerminalWindowAttr *window = wp->data;
      GSList *tp;

      g_print ("W");
      print_string ("display", window->display);
      print_string ("geometry", window->geometry);
      print_string ("role", window->role);
      g_print ("|workspace=%d", window->workspace);
      print_string ("startup_id", window->startup_id);
      print_string ("sm_client_id", window->sm_client_id);
      print_string ("icon", window->icon);
      print_string ("font", window->font);
      g_print ("|drop_down=%u|fullscreen=%u|maximize=%u|minimize=%u"
               "|reuse_last_window=%u|menubar=%d|borders=%d|toolbar=%d"
               "|scrollbar=%d|zoom=%d\n",
               window->drop_down,
               window->fullscreen,
               window->maximize,
               window->minimize,
               window->reuse_last_window,
               window->menubar,
               window->borders,
               window->toolbar,
               window->scrollbar,
               window->zoom);

      for (tp = window->tabs; tp != NULL; tp = tp->next)
        {
          TerminalTabAttr *tab = tp->data;

          g_print ("T");
          print_command (tab->command);
          print_string ("directory", tab->directory);
          print_string ("title", tab->title);
          print_string ("initial_title", tab->initial_title);
          print_string ("color_text", tab->color_text);
          print_string ("color_bg", tab->color_bg);
          print_string ("color_title", tab->color_title);
          g_print ("|dynamic_title_mode=%d|position=%d|hold=%u|active=%u\n",
                   tab->dynamic_title_mode,
                   tab->position,
                   tab->hold,
                   tab->active);
        }
    }

  g_slist_free_full (windows, (GDestroyNotify) terminal_window_attr_free);
  return EXIT_SUCCESS;
}
