/*
 * Reports the frozen terminal screen's VTE search state.
 *
 * The screen helpers are public, but their VTE terminal is private to the
 * screen. Including the frozen implementation lets this probe inspect the
 * state those helpers leave at the VTE boundary without reimplementing a
 * screen.
 */

#include <gtk/gtk.h>
#include <stdio.h>

#include "terminal-screen.c"

#define SEARCH_FLAGS (PCRE2_UTF | PCRE2_NO_UTF_CHECK | PCRE2_MULTILINE)


static const char *
bool_text (gboolean value)
{
  return value ? "true" : "false";
}


int
main (int argc,
      char **argv)
{
  GError *error = NULL;
  TerminalScreen *screen;
  GtkWidget *window;
  VteRegex *regex;

  gtk_init (&argc, &argv);

  screen = g_object_new (TERMINAL_TYPE_SCREEN, NULL);
  window = g_object_ref_sink (gtk_window_new (GTK_WINDOW_TOPLEVEL));
  gtk_container_add (GTK_CONTAINER (window), GTK_WIDGET (screen));
  gtk_widget_realize (window);

  printf ("initial\t%s\t%s\n",
          bool_text (terminal_screen_search_has_gregex (screen)),
          bool_text (vte_terminal_search_get_wrap_around (VTE_TERMINAL (screen->terminal))));

  regex = vte_regex_new_for_search ("needle", -1, SEARCH_FLAGS, &error);
  if (regex == NULL)
    {
      g_printerr ("create search regular expression: %s\n", error->message);
      g_clear_error (&error);
      return 1;
    }

  terminal_screen_search_set_gregex (screen, regex, TRUE);
  printf ("configured\t%s\t%s\n",
          bool_text (terminal_screen_search_has_gregex (screen)),
          bool_text (vte_terminal_search_get_wrap_around (VTE_TERMINAL (screen->terminal))));

  terminal_screen_search_find_next (screen);
  terminal_screen_search_find_previous (screen);
  printf ("moves\tcalled\n");

  terminal_screen_reset (screen, FALSE);
  printf ("reset-keeps\t%s\t%s\n",
          bool_text (terminal_screen_search_has_gregex (screen)),
          bool_text (vte_terminal_search_get_wrap_around (VTE_TERMINAL (screen->terminal))));

  terminal_screen_reset (screen, TRUE);
  printf ("reset-clears\t%s\t%s\n",
          bool_text (terminal_screen_search_has_gregex (screen)),
          bool_text (vte_terminal_search_get_wrap_around (VTE_TERMINAL (screen->terminal))));

  terminal_screen_search_set_gregex (screen, NULL, FALSE);
  printf ("explicit-clear\t%s\t%s\n",
          bool_text (terminal_screen_search_has_gregex (screen)),
          bool_text (vte_terminal_search_get_wrap_around (VTE_TERMINAL (screen->terminal))));

  vte_regex_unref (regex);
  g_object_unref (window);
  return 0;
}
