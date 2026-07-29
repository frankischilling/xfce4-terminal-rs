#define _GNU_SOURCE

#include <dlfcn.h>
#include <gtk/gtk.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

/*
 * Observe the arguments passed by each executable at its native library
 * boundary. The accelerator hook calls through to GTK before printing the
 * captured contract and exiting, which keeps the frozen preferences dialog
 * from blocking the noninteractive test.
 */
typedef void (*TextdomainFunc) (const gchar *,
                                const gchar *,
                                const gchar *);
typedef void (*AccelMapLoadFunc) (const gchar *);

static gchar gettext_domain[128];
static gchar locale_directory[4096];
static gchar gettext_charset[128];

static void
print_accelerator (gpointer data,
                   const gchar *accel_path,
                   guint accel_key,
                   GdkModifierType accel_mods,
                   gboolean changed)
{
  gchar *accelerator = gtk_accelerator_name (accel_key, accel_mods);

  (void) data;
  (void) changed;
  dprintf (STDOUT_FILENO, "accelerator\t%s\t%s\n",
           accel_path, accelerator);
  g_free (accelerator);
}

void
xfce_textdomain (const gchar *domain,
                 const gchar *locale_dir,
                 const gchar *charset)
{
  TextdomainFunc next = (TextdomainFunc) dlsym (RTLD_NEXT, "xfce_textdomain");

  if (next == NULL)
    {
      dprintf (STDERR_FILENO, "cannot resolve xfce_textdomain\n");
      _exit (125);
    }
  g_strlcpy (gettext_domain, domain, sizeof (gettext_domain));
  g_strlcpy (locale_directory, locale_dir, sizeof (locale_directory));
  g_strlcpy (gettext_charset, charset, sizeof (gettext_charset));
  next (domain, locale_dir, charset);
}

void
gtk_accel_map_load (const gchar *file_name)
{
  AccelMapLoadFunc next = (AccelMapLoadFunc) dlsym (RTLD_NEXT, "gtk_accel_map_load");

  if (next == NULL)
    {
      dprintf (STDERR_FILENO, "cannot resolve gtk_accel_map_load\n");
      _exit (125);
    }
  next (file_name);

  dprintf (STDOUT_FILENO, "gettext-domain\t%s\n", gettext_domain);
  dprintf (STDOUT_FILENO, "locale-directory\t%s\n", locale_directory);
  dprintf (STDOUT_FILENO, "gettext-charset\t%s\n", gettext_charset);
  dprintf (STDOUT_FILENO, "accelerator-file\t%s\n", file_name);
  gtk_accel_map_foreach (NULL, print_accelerator);
  _exit (0);
}
