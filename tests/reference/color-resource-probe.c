#include <glib.h>
#include <libxfce4util/libxfce4util.h>


static void
print_matches (XfceResourceType type,
               const gchar *label)
{
  gchar **matches;
  guint n;

  matches = xfce_resource_match (type,
                                 "xfce4/terminal/colorschemes/*",
                                 TRUE);
  for (n = 0; matches[n] != NULL; ++n)
    {
      gchar *path = xfce_resource_lookup (type, matches[n]);

      if (path != NULL)
        {
          g_print ("%s\t%s\n", label, path);
          g_free (path);
        }
    }
  g_strfreev (matches);
}


int
main (void)
{
  print_matches (XFCE_RESOURCE_DATA, "data");
  print_matches (XFCE_RESOURCE_CONFIG, "config");
  return 0;
}
