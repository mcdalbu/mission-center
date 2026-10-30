# RPM validation — 2026-10-29

Tested on Fedora 44 x86_64 with GTK 4.22 and GNOME 50.

- `rpmbuild -ba` completed, producing the application RPM, source RPM,
  debuginfo, and debugsource packages.
- All four CPU presentation unit tests passed: EPP names/aliases/custom values,
  CPU list parsing, frequency formatting/unavailable readings, and sysfs topology.
- Both Meson desktop/schema tests passed.
- Binary and source RPM digest checks passed (`rpm -K`). These are unsigned
  local packages; a successful digest check does not imply a trusted signature.
- The actual application RPM was extracted and launched in an isolated GNOME
  Wayland session with its matching gatherer and resources. A GTK observer
  confirmed eight visible CPU badges with live GHz samples: four P-cores and
  four E-cores on an Intel Core Ultra 7 256V. The native EPP widget reported
  `128 · Balanced`. The captured window was inspected visually.
- The observer was only a test harness. The packaged application implements
  these features directly and requires no preload library.

This validates application startup and the changed CPU UI, not every upstream
feature. The baseline is Slimbook-Team's 0.6.1, not the newer 1.2.0 Flatpak.
The package was not installed over the host's existing Flatpak.
