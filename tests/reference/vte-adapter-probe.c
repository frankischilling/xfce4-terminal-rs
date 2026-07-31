/*
 * Reports the frozen terminal widget's VTE link registration and copy action.
 *
 * The widget owns both the preference notification and the VTE match tags. The
 * probe includes the frozen implementation so it can count those tags without
 * reproducing the update loop outside the reference program.
 */

#include <gtk/gtk.h>
#include <stdio.h>

#include "terminal-widget.c"


static guint
registered_pattern_count (TerminalWidget *widget)
{
  guint count = 0;
  guint i;

  for (i = 0; i < G_N_ELEMENTS (widget->regex_tags); i++)
    if (widget->regex_tags[i] != -1)
      count++;

  return count;
}


int
main (int argc,
      char **argv)
{
  TerminalWidget *widget;
  GtkWidget *window;
  GtkWidget *item;
  GdkDisplay *display;
  TerminalPreferences *preferences;
  gchar *primary;
  gchar *clipboard;

  gtk_init (&argc, &argv);

  preferences = terminal_preferences_get ();
  g_object_set (preferences, "misc-highlight-urls", FALSE, NULL);
  g_object_unref (preferences);

  widget = g_object_new (TERMINAL_TYPE_WIDGET, NULL);
  window = gtk_window_new (GTK_WINDOW_TOPLEVEL);
  gtk_container_add (GTK_CONTAINER (window), GTK_WIDGET (widget));
  g_object_ref_sink (window);

  printf ("initial-highlighted-patterns\t%u\n", registered_pattern_count (widget));

  g_object_set (widget->preferences, "misc-highlight-urls", TRUE, NULL);
  printf ("enabled-patterns\t%u\n", registered_pattern_count (widget));

  item = g_object_ref_sink (gtk_menu_item_new ());
  g_object_set_data_full (G_OBJECT (item), "terminal-widget-link",
                          g_strdup ("mailto:user@example.com"), g_free);
  terminal_widget_context_menu_copy (widget, item);

  display = gtk_widget_get_display (GTK_WIDGET (widget));
  primary = gtk_clipboard_wait_for_text (
    gtk_clipboard_get_for_display (display, GDK_SELECTION_PRIMARY));
  clipboard = gtk_clipboard_wait_for_text (
    gtk_clipboard_get_for_display (display, GDK_SELECTION_CLIPBOARD));
  printf ("primary\t%s\n", primary != NULL ? primary : "<none>");
  printf ("clipboard\t%s\n", clipboard != NULL ? clipboard : "<none>");
  g_free (primary);
  g_free (clipboard);
  g_object_unref (item);

  g_object_set (widget->preferences, "misc-highlight-urls", FALSE, NULL);
  printf ("highlight-disabled\t%u\n", registered_pattern_count (widget));

  g_object_unref (window);
  return 0;
}
