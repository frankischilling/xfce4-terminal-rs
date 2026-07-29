/*
 * Writes the screen-model contract of the frozen C terminal screen.
 *
 * Titles, paste safety, the working directory, and the colors handed to VTE
 * are decided by file-private helpers, so this probe includes the screen
 * implementation instead of linking against its object file. The remaining
 * frozen objects are linked unchanged.
 *
 * VTE color setters are replaced here so the report records the values the
 * frozen update would pass, without depending on VTE to echo them back. The
 * real library symbols stay unused for those calls because the included
 * screen source resolves to the definitions in this file.
 *
 * The report goes to a named file rather than to standard output, because the
 * display and session-bus wrappers the probe runs under add their own output.
 * It is written with stdio so that candidates keep their exact bytes.
 */

#include <gtk/gtk.h>
#include <stdio.h>
#include <string.h>

#include "terminal-screen.c"


static FILE *report = NULL;
static gchar *forced_window_title = NULL;
static gchar *forced_directory_uri = NULL;

static gboolean colors_default = FALSE;
static gboolean has_reported_fg = FALSE;
static gboolean has_reported_bg = FALSE;
static GdkRGBA reported_fg;
static GdkRGBA reported_bg;
static GdkRGBA reported_palette[16];
static gsize reported_palette_size = 0;
static gboolean has_cursor_fg = FALSE;
static gboolean has_cursor = FALSE;
static gboolean cursor_fg_cleared = FALSE;
static gboolean cursor_cleared = FALSE;
static GdkRGBA reported_cursor_fg;
static GdkRGBA reported_cursor;
static gboolean has_selection_fg = FALSE;
static gboolean has_selection_bg = FALSE;
static gboolean selection_fg_cleared = FALSE;
static gboolean selection_bg_cleared = FALSE;
static GdkRGBA reported_selection_fg;
static GdkRGBA reported_selection_bg;
static gboolean has_bold = FALSE;
static gboolean bold_cleared = FALSE;
static GdkRGBA reported_bold;
static gboolean bold_is_bright_set = FALSE;
static gboolean reported_bold_is_bright = FALSE;


static void
reset_color_report (void)
{
  colors_default = FALSE;
  reported_palette_size = 0;
  has_reported_fg = FALSE;
  has_reported_bg = FALSE;
  has_cursor_fg = FALSE;
  has_cursor = FALSE;
  cursor_fg_cleared = FALSE;
  cursor_cleared = FALSE;
  has_selection_fg = FALSE;
  has_selection_bg = FALSE;
  selection_fg_cleared = FALSE;
  selection_bg_cleared = FALSE;
  has_bold = FALSE;
  bold_cleared = FALSE;
  bold_is_bright_set = FALSE;
  reported_bold_is_bright = FALSE;
}


void
vte_terminal_set_colors (VteTerminal *terminal,
                         const GdkRGBA *foreground,
                         const GdkRGBA *background,
                         const GdkRGBA *palette,
                         gsize palette_size)
{
  gsize i;

  (void) terminal;
  colors_default = FALSE;
  has_reported_fg = foreground != NULL;
  if (foreground != NULL)
    reported_fg = *foreground;
  has_reported_bg = background != NULL;
  if (background != NULL)
    reported_bg = *background;
  reported_palette_size = MIN (palette_size, 16);
  for (i = 0; i < reported_palette_size; i++)
    reported_palette[i] = palette[i];
}


void
vte_terminal_set_default_colors (VteTerminal *terminal)
{
  (void) terminal;
  colors_default = TRUE;
  reported_palette_size = 0;
  has_reported_fg = FALSE;
  has_reported_bg = FALSE;
}


void
vte_terminal_set_color_cursor_foreground (VteTerminal *terminal,
                                          const GdkRGBA *color)
{
  (void) terminal;
  if (color == NULL)
    {
      cursor_fg_cleared = TRUE;
      has_cursor_fg = FALSE;
    }
  else
    {
      cursor_fg_cleared = FALSE;
      has_cursor_fg = TRUE;
      reported_cursor_fg = *color;
    }
}


void
vte_terminal_set_color_cursor (VteTerminal *terminal,
                               const GdkRGBA *color)
{
  (void) terminal;
  if (color == NULL)
    {
      cursor_cleared = TRUE;
      has_cursor = FALSE;
    }
  else
    {
      cursor_cleared = FALSE;
      has_cursor = TRUE;
      reported_cursor = *color;
    }
}


void
vte_terminal_set_color_highlight_foreground (VteTerminal *terminal,
                                             const GdkRGBA *color)
{
  (void) terminal;
  if (color == NULL)
    {
      selection_fg_cleared = TRUE;
      has_selection_fg = FALSE;
    }
  else
    {
      selection_fg_cleared = FALSE;
      has_selection_fg = TRUE;
      reported_selection_fg = *color;
    }
}


void
vte_terminal_set_color_highlight (VteTerminal *terminal,
                                  const GdkRGBA *color)
{
  (void) terminal;
  if (color == NULL)
    {
      selection_bg_cleared = TRUE;
      has_selection_bg = FALSE;
    }
  else
    {
      selection_bg_cleared = FALSE;
      has_selection_bg = TRUE;
      reported_selection_bg = *color;
    }
}


void
vte_terminal_set_color_bold (VteTerminal *terminal,
                             const GdkRGBA *color)
{
  (void) terminal;
  if (color == NULL)
    {
      bold_cleared = TRUE;
      has_bold = FALSE;
    }
  else
    {
      bold_cleared = FALSE;
      has_bold = TRUE;
      reported_bold = *color;
    }
}


void
vte_terminal_set_bold_is_bright (VteTerminal *terminal,
                                 gboolean bold_is_bright)
{
  (void) terminal;
  bold_is_bright_set = TRUE;
  reported_bold_is_bright = bold_is_bright;
}


const char *
vte_terminal_get_window_title (VteTerminal *terminal)
{
  (void) terminal;
  return forced_window_title;
}


const char *
vte_terminal_get_current_directory_uri (VteTerminal *terminal)
{
  (void) terminal;
  return forced_directory_uri;
}


static void
write_rgba (const char *name,
            const GdkRGBA *color)
{
  fprintf (report, "color\t%s\t%.6f\t%.6f\t%.6f\t%.6f\n",
           name, color->red, color->green, color->blue, color->alpha);
}


static gchar *
unescape_field (const gchar *field)
{
  GString *string;
  const gchar *p;

  if (g_strcmp0 (field, "-") == 0)
    return NULL;

  string = g_string_new (NULL);
  for (p = field; *p != '\0'; p++)
    {
      if (p[0] == '\\' && p[1] != '\0')
        {
          switch (p[1])
            {
            case 'n':
              g_string_append_c (string, '\n');
              p++;
              break;
            case 'r':
              g_string_append_c (string, '\r');
              p++;
              break;
            case 't':
              g_string_append_c (string, '\t');
              p++;
              break;
            case '\\':
              g_string_append_c (string, '\\');
              p++;
              break;
            default:
              g_string_append_c (string, *p);
              break;
            }
        }
      else
        g_string_append_c (string, *p);
    }

  return g_string_free (string, FALSE);
}


static TerminalTitle
parse_title_mode (const gchar *name)
{
  if (g_strcmp0 (name, "TERMINAL_TITLE_REPLACE") == 0)
    return TERMINAL_TITLE_REPLACE;
  if (g_strcmp0 (name, "TERMINAL_TITLE_PREPEND") == 0)
    return TERMINAL_TITLE_PREPEND;
  if (g_strcmp0 (name, "TERMINAL_TITLE_APPEND") == 0)
    return TERMINAL_TITLE_APPEND;
  if (g_strcmp0 (name, "TERMINAL_TITLE_HIDE") == 0)
    return TERMINAL_TITLE_HIDE;
  g_error ("unknown title mode %s", name);
}


static void
set_window_title (const gchar *title)
{
  g_free (forced_window_title);
  forced_window_title = g_strdup (title);
}


static void
set_directory_uri (const gchar *uri)
{
  g_free (forced_directory_uri);
  forced_directory_uri = g_strdup (uri);
}


static void
report_parse_title (TerminalScreen *screen,
                    guint index,
                    gchar **fields)
{
  gchar *directory;
  gchar *vte_title;
  gchar *template;
  gchar *parsed;

  if (g_strv_length (fields) != 5)
    g_error ("title-parse needs four fields after the kind");

  screen->session_id = (guint) g_ascii_strtoull (fields[1], NULL, 10);
  directory = unescape_field (fields[2]);
  vte_title = unescape_field (fields[3]);
  template = unescape_field (fields[4]);

  if (directory != NULL)
    terminal_screen_set_working_directory (screen, directory);
  set_window_title (vte_title);

  parsed = terminal_screen_parse_title (screen, template);
  fprintf (report, "title-parse\t%u\t%s\n", index, parsed);
  g_free (parsed);
  g_free (directory);
  g_free (vte_title);
  g_free (template);
}


static void
report_title (TerminalScreen *screen,
              guint index,
              gchar **fields)
{
  gchar *custom;
  gchar *initial;
  gchar *preference_initial;
  gchar *directory;
  gchar *vte_title;
  gchar *title;
  TerminalTitle mode;

  if (g_strv_length (fields) != 8)
    g_error ("title needs seven fields after the kind");

  custom = unescape_field (fields[1]);
  initial = unescape_field (fields[2]);
  preference_initial = unescape_field (fields[3]);
  mode = parse_title_mode (fields[4]);
  screen->session_id = (guint) g_ascii_strtoull (fields[5], NULL, 10);
  directory = unescape_field (fields[6]);
  vte_title = unescape_field (fields[7]);

  g_free (screen->custom_title);
  screen->custom_title = custom;
  g_free (screen->initial_title);
  screen->initial_title = initial;
  screen->dynamic_title_mode = mode;
  g_object_set (G_OBJECT (screen->preferences),
                "title-initial", preference_initial != NULL ? preference_initial : "",
                NULL);
  if (directory != NULL)
    terminal_screen_set_working_directory (screen, directory);
  set_window_title (vte_title);

  title = terminal_screen_get_title (screen);
  fprintf (report, "title\t%u\t%s\n", index, title);
  g_free (title);
  g_free (preference_initial);
  g_free (directory);
  g_free (vte_title);
}


static void
report_paste (guint index,
              gchar **fields)
{
  gchar *text;

  if (g_strv_length (fields) != 2)
    g_error ("paste needs one field after the kind");

  text = unescape_field (fields[1]);
  fprintf (report, "paste\t%u\t%s\n",
           index, terminal_screen_is_text_unsafe (text) ? "unsafe" : "safe");
  g_free (text);
}


static void
report_cwd (TerminalScreen *screen,
            guint index,
            gchar **fields)
{
  gchar *stored;
  gchar *uri;
  const gchar *directory;

  if (g_strv_length (fields) != 3)
    g_error ("cwd needs two fields after the kind");

  stored = unescape_field (fields[1]);
  uri = unescape_field (fields[2]);

  /* A fresh screen starts with pid -1, so only the URI path and the stored
   * directory are under test here. Process-cwd coverage stays with the unit
   * tests, which supply that value as an explicit input. */
  screen->pid = -1;
  if (stored != NULL)
    terminal_screen_set_working_directory (screen, stored);
  set_directory_uri (uri);

  directory = terminal_screen_get_working_directory (screen);
  fprintf (report, "cwd\t%u\t%s\n", index, directory != NULL ? directory : "");
  g_free (stored);
  g_free (uri);
}


static void
report_colors (TerminalScreen *screen,
               guint index,
               gchar **fields)
{
  gchar *palette;
  gchar *foreground;
  gchar *background;
  gchar *custom_fg;
  gchar *custom_bg;
  gchar *cursor_fg;
  gchar *cursor;
  gchar *selection;
  gchar *selection_bg;
  gchar *bold;
  gsize i;

  if (g_strv_length (fields) != 17)
    g_error ("colors needs sixteen fields after the kind");

  palette = unescape_field (fields[1]);
  foreground = unescape_field (fields[2]);
  background = unescape_field (fields[3]);
  custom_fg = unescape_field (fields[10]);
  custom_bg = unescape_field (fields[11]);
  cursor_fg = unescape_field (fields[12]);
  cursor = unescape_field (fields[13]);
  selection = unescape_field (fields[14]);
  selection_bg = unescape_field (fields[15]);
  bold = unescape_field (fields[16]);

  g_free (screen->custom_fg_color);
  screen->custom_fg_color = custom_fg;
  g_free (screen->custom_bg_color);
  screen->custom_bg_color = custom_bg;
  screen->has_random_bg_color = 0;

  g_object_set (G_OBJECT (screen->preferences),
                "color-palette", palette,
                "color-foreground", foreground,
                "color-background", background,
                "color-use-theme", g_strcmp0 (fields[4], "true") == 0,
                "color-background-vary", g_strcmp0 (fields[5], "true") == 0,
                "color-cursor-use-default", g_strcmp0 (fields[6], "true") == 0,
                "color-selection-use-default", g_strcmp0 (fields[7], "true") == 0,
                "color-bold-use-default", g_strcmp0 (fields[8], "true") == 0,
                "color-bold-is-bright", g_strcmp0 (fields[9], "true") == 0,
                "color-cursor-foreground", cursor_fg,
                "color-cursor", cursor,
                "color-selection", selection,
                "color-selection-background", selection_bg,
                "color-bold", bold,
                NULL);

  reset_color_report ();
  terminal_screen_update_colors (screen);

  fprintf (report, "colors\t%u\t%s\n",
           index, colors_default ? "default" : "palette");
  if (has_reported_fg)
    write_rgba ("fg", &reported_fg);
  else
    fprintf (report, "color\tfg\t-\n");
  if (has_reported_bg)
    write_rgba ("bg", &reported_bg);
  else
    fprintf (report, "color\tbg\t-\n");
  for (i = 0; i < reported_palette_size; i++)
    {
      gchar name[32];
      g_snprintf (name, sizeof (name), "palette-%zu", i);
      write_rgba (name, &reported_palette[i]);
    }
  if (has_cursor_fg)
    write_rgba ("cursor-fg", &reported_cursor_fg);
  else if (cursor_fg_cleared)
    fprintf (report, "color\tcursor-fg\t-\n");
  if (has_cursor)
    write_rgba ("cursor", &reported_cursor);
  else if (cursor_cleared)
    fprintf (report, "color\tcursor\t-\n");
  if (has_selection_fg)
    write_rgba ("selection-fg", &reported_selection_fg);
  else if (selection_fg_cleared)
    fprintf (report, "color\tselection-fg\t-\n");
  if (has_selection_bg)
    write_rgba ("selection-bg", &reported_selection_bg);
  else if (selection_bg_cleared)
    fprintf (report, "color\tselection-bg\t-\n");
  if (has_bold)
    write_rgba ("bold", &reported_bold);
  else if (bold_cleared)
    fprintf (report, "color\tbold\t-\n");
  if (bold_is_bright_set)
    fprintf (report, "bold-is-bright\t%u\t%s\n",
             index, reported_bold_is_bright ? "true" : "false");

  g_free (palette);
  g_free (foreground);
  g_free (background);
  g_free (cursor_fg);
  g_free (cursor);
  g_free (selection);
  g_free (selection_bg);
  g_free (bold);
}


int
main (int argc,
      char **argv)
{
  TerminalScreen *screen;
  GtkWidget *window;
  gchar *fixtures;
  gchar **scenarios;
  guint index;

  gtk_init (&argc, &argv);

  if (argc != 3)
    {
      g_printerr ("usage: %s FIXTURE_FILE REPORT_FILE\n", argv[0]);
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

  scenarios = g_strsplit (fixtures, "\n", -1);
  for (index = 0; scenarios[index] != NULL; index++)
    {
      gchar **fields;

      if (scenarios[index][0] == '#'
          || (scenarios[index + 1] == NULL && scenarios[index][0] == '\0'))
        continue;

      /* A fresh screen keeps tab colors from leaking between scenarios. Window
       * titles and directory URIs come from the interposed VTE getters below,
       * which are reset for every scenario. Colors also read the toplevel
       * style context, so the screen lives inside a realized window. */
      set_window_title (NULL);
      set_directory_uri (NULL);
      screen = g_object_new (TERMINAL_TYPE_SCREEN, NULL);
      window = g_object_ref_sink (gtk_window_new (GTK_WINDOW_TOPLEVEL));
      gtk_container_add (GTK_CONTAINER (window), GTK_WIDGET (screen));
      gtk_widget_realize (window);

      fprintf (report, "scenario\t%u\t%s\n", index, scenarios[index]);
      fields = g_strsplit (scenarios[index], "\t", -1);

      if (g_strcmp0 (fields[0], "title-parse") == 0)
        report_parse_title (screen, index, fields);
      else if (g_strcmp0 (fields[0], "title") == 0)
        report_title (screen, index, fields);
      else if (g_strcmp0 (fields[0], "paste") == 0)
        report_paste (index, fields);
      else if (g_strcmp0 (fields[0], "cwd") == 0)
        report_cwd (screen, index, fields);
      else if (g_strcmp0 (fields[0], "colors") == 0)
        report_colors (screen, index, fields);
      else
        {
          g_printerr ("unknown scenario on line %u\n", index);
          return 2;
        }

      g_strfreev (fields);
      g_object_unref (window);
    }

  g_strfreev (scenarios);
  g_free (fixtures);
  return fclose (report) == 0 ? 0 : 2;
}
