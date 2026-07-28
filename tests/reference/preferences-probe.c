/*
 * Prints the public preference definition contract from the frozen C class.
 *
 * The output is deliberately line based so the Rust parity test can compare
 * it without depending on GObject implementation details.
 */

#include <glib-object.h>
#include <gtk/gtk.h>

#include "terminal-preferences.h"


static void
print_enum_values (GType type)
{
  GEnumClass *enum_class;
  guint n;

  enum_class = g_type_class_ref (type);
  for (n = 0; n < enum_class->n_values; ++n)
    {
      if (n > 0)
        g_print (",");
      g_print ("%s", enum_class->values[n].value_name);
    }
  g_type_class_unref (enum_class);
}


static void
print_property (GParamSpec *spec)
{
  const GValue *default_value;
  GType value_type;

  value_type = G_PARAM_SPEC_VALUE_TYPE (spec);
  default_value = g_param_spec_get_default_value (spec);

  g_print ("%s\t", g_param_spec_get_name (spec));
  if (value_type == G_TYPE_BOOLEAN)
    g_print ("boolean\t%s\t\t",
             g_value_get_boolean (default_value) ? "true" : "false");
  else if (value_type == G_TYPE_UINT)
    {
      GParamSpecUInt *uint_spec = G_PARAM_SPEC_UINT (spec);
      g_print ("uint\t%u\t%u:%u\t",
               g_value_get_uint (default_value),
               uint_spec->minimum,
               uint_spec->maximum);
    }
  else if (value_type == G_TYPE_DOUBLE)
    {
      GParamSpecDouble *double_spec = G_PARAM_SPEC_DOUBLE (spec);
      gchar default_buffer[G_ASCII_DTOSTR_BUF_SIZE];
      gchar minimum_buffer[G_ASCII_DTOSTR_BUF_SIZE];
      gchar maximum_buffer[G_ASCII_DTOSTR_BUF_SIZE];

      g_ascii_dtostr (default_buffer, sizeof (default_buffer),
                      g_value_get_double (default_value));
      g_ascii_dtostr (minimum_buffer, sizeof (minimum_buffer),
                      double_spec->minimum);
      g_ascii_dtostr (maximum_buffer, sizeof (maximum_buffer),
                      double_spec->maximum);
      g_print ("double\t%s\t%s:%s\t",
               default_buffer, minimum_buffer, maximum_buffer);
    }
  else if (value_type == G_TYPE_STRING)
    {
      const gchar *value = g_value_get_string (default_value);
      g_print ("string\t%s\t\t", value != NULL ? value : "<null>");
    }
  else if (G_TYPE_IS_ENUM (value_type))
    {
      GEnumClass *enum_class = g_type_class_ref (value_type);
      GEnumValue *enum_value =
        g_enum_get_value (enum_class, g_value_get_enum (default_value));

      g_print ("enum:%s\t%s\t", g_type_name (value_type),
               enum_value->value_name);
      print_enum_values (value_type);
      g_print ("\t");
      g_type_class_unref (enum_class);
    }
  else
    {
      g_error ("unsupported preference type %s", g_type_name (value_type));
    }

  g_print ("%s\n", g_param_spec_get_blurb (spec));
}

static void
print_value (GObject *preferences,
             GParamSpec *spec)
{
  GValue value = G_VALUE_INIT;
  GType value_type;

  value_type = G_PARAM_SPEC_VALUE_TYPE (spec);
  g_value_init (&value, value_type);
  g_object_get_property (preferences, g_param_spec_get_name (spec), &value);

  g_print ("%s\t", g_param_spec_get_name (spec));
  if (value_type == G_TYPE_BOOLEAN)
    g_print ("%s", g_value_get_boolean (&value) ? "true" : "false");
  else if (value_type == G_TYPE_UINT)
    g_print ("%u", g_value_get_uint (&value));
  else if (value_type == G_TYPE_DOUBLE)
    {
      gchar buffer[G_ASCII_DTOSTR_BUF_SIZE];
      g_print ("%s", g_ascii_dtostr (buffer, sizeof (buffer),
                                     g_value_get_double (&value)));
    }
  else if (value_type == G_TYPE_STRING)
    {
      const gchar *string = g_value_get_string (&value);
      g_print ("%s", string != NULL ? string : "<null>");
    }
  else if (G_TYPE_IS_ENUM (value_type))
    {
      GEnumClass *enum_class = g_type_class_ref (value_type);
      GEnumValue *enum_value =
        g_enum_get_value (enum_class, g_value_get_enum (&value));
      g_print ("%s", enum_value->value_name);
      g_type_class_unref (enum_class);
    }
  else
    {
      g_error ("unsupported preference type %s", g_type_name (value_type));
    }
  g_print ("\n");
  g_value_unset (&value);
}


int
main (int argc,
      char **argv)
{
  GObjectClass *preferences_class;
  TerminalPreferences *preferences;
  GParamSpec **properties;
  guint n_properties;
  guint n;

  preferences_class = g_type_class_ref (terminal_preferences_get_type ());
  properties = g_object_class_list_properties (preferences_class,
                                                &n_properties);
  if (argc == 2 && g_strcmp0 (argv[1], "--values") == 0)
    {
      preferences = terminal_preferences_get ();
      for (n = 0; n < n_properties; ++n)
        print_value (G_OBJECT (preferences), properties[n]);
      g_object_unref (preferences);
    }
  else
    {
      for (n = 0; n < n_properties; ++n)
        print_property (properties[n]);
    }

  g_free (properties);
  g_type_class_unref (preferences_class);
  return 0;
}
