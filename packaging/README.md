# Fedora RPM: CPU details fork

This fork is based on **Slimbook-Team/mission-center 0.6.1**. It is not based
on the newer 1.2.0 Flathub build. The native code adds:

- friendly numeric EPP labels, including kernel aliases and exact custom values;
- current frequency on each individual CPU utilization graph;
- P-core/E-core labels from the kernel's Intel PMU CPU lists;
- a normal RPM and source RPM, without an LD_PRELOAD adapter.

The frequency is the kernel-reported `scaling_cur_freq` sample, refreshed at
Mission Center's existing reading interval while those labels are mapped.
It is not a continuous measurement. Unavailable readings show a dash. On
hardware without unambiguous P/E metadata the type is omitted, never inferred
from clock speed. No power settings are written and no new timer is added.

## Build on Fedora 44 x86_64

```sh
sudo dnf install rust cargo gcc gcc-c++ meson ninja-build patch rpm-build \
  blueprint-compiler gettext python3-gobject desktop-file-utils \
  gtk4-devel libadwaita-devel libdrm-devel mesa-libEGL-devel mesa-libgbm-devel \
  systemd-devel dbus-devel mesa-libGL-devel
./packaging/build-rpm.sh
```

Preparation downloads the locked Cargo crates, vendors both Rust dependency
sets, and verifies the pinned NVTOP source archive. The source RPM contains
those inputs. Compilation then runs with Cargo offline and a local native
source cache; it does not require network access during the RPM build.

Build output:

```text
_rpm/RPMS/x86_64/mission-center-custom-*.rpm
_rpm/SRPMS/mission-center-custom-*.src.rpm
```

The spec limits Cargo parallelism to two jobs and disables release LTO to
keep memory use manageable. It runs the CPU presentation unit tests and
Meson's desktop/schema checks. These are local custom packages, not official
Fedora distribution packages.

## Install and run

Review the package and install the binary RPM (not the source RPM):

```sh
rpm -qip _rpm/RPMS/x86_64/mission-center-custom-*.rpm
sudo dnf install ./_rpm/RPMS/x86_64/mission-center-custom-*.rpm
missioncenter
```

Quit an already running Mission Center first. This base uses the same
application identity as upstream, so running a Flatpak instance and the native
fork at the same time can activate the existing instance instead of the fork.
The RPM conflicts with other native packages named mission-center/missioncenter;
it does not uninstall the separately installed Flatpak.

Choose CPU → individual processors (all CPUs) to see the labels. Graphs still
plot utilization; the badge reports CPU identity and sampled frequency.

If the earlier local display adapter was enabled for the Flatpak, it only
applies to Flatpak launches. The native RPM does not need it. Its reversal
instructions remain in the local integration guide; no kernel patch is needed.

## Rebuild from the source RPM

```sh
rpmbuild --rebuild mission-center-custom-*.src.rpm
```

Build dependencies must be installed first. The CPU helper tests can also run
without building GTK bindings:

```sh
rustc --edition=2021 --test src/performance_page/cpu_presentation.rs -o /tmp/cpu-presentation-tests
/tmp/cpu-presentation-tests
```

Remove the RPM with `sudo dnf remove mission-center-custom`. This removes the
native app; it does not change the power-profile services or Flatpak installation.
