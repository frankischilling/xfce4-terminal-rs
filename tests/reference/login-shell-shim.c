/*
 * Puts a host's login shells under the control of the comparison.
 *
 * The frozen reference and the Rust candidate both read the password database
 * and test files for execute permission through libc, so preloading this
 * library into either of them lets one comparison cover a host that has no
 * usable shell at all. Only the two questions the shell search asks are
 * answered here, and only for the paths the test names; everything else is
 * passed through to libc.
 *
 * XFCE4_TERMINAL_PROBE_PW_SHELL replaces the shell of every password entry, or
 * removes it when the value is empty. XFCE4_TERMINAL_PROBE_DENY_EXEC holds
 * colon separated paths that stop looking executable.
 */

#define _GNU_SOURCE

#include <dlfcn.h>
#include <errno.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>


static void *
next_symbol (const char *name)
{
  void *symbol = dlsym (RTLD_NEXT, name);

  if (symbol == NULL)
    {
      dprintf (STDERR_FILENO, "cannot resolve %s\n", name);
      _exit (125);
    }
  return symbol;
}


static int
listed (const char *list,
        const char *path)
{
  size_t length = strlen (path);
  const char *entry = list;

  while (*entry != '\0')
    {
      const char *end = strchr (entry, ':');
      size_t entry_length = end != NULL ? (size_t) (end - entry) : strlen (entry);

      if (entry_length == length && strncmp (entry, path, length) == 0)
        return 1;
      if (end == NULL)
        break;
      entry = end + 1;
    }

  return 0;
}


struct passwd *
getpwuid (uid_t uid)
{
  struct passwd *(*next) (uid_t) = next_symbol ("getpwuid");
  const char *shell = getenv ("XFCE4_TERMINAL_PROBE_PW_SHELL");
  struct passwd *entry = next (uid);

  /* The entry lives in storage the C library owns and reuses, which is where
   * the caller reads the shell from, so the replacement is written there. */
  if (entry != NULL && shell != NULL)
    entry->pw_shell = shell[0] != '\0' ? (char *) shell : NULL;

  return entry;
}


int
access (const char *path,
        int mode)
{
  int (*next) (const char *, int) = next_symbol ("access");
  const char *denied = getenv ("XFCE4_TERMINAL_PROBE_DENY_EXEC");

  if ((mode & X_OK) != 0 && denied != NULL && path != NULL && listed (denied, path))
    {
      errno = EACCES;
      return -1;
    }

  return next (path, mode);
}
