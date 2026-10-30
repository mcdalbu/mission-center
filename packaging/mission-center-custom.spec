Name:           mission-center-custom
Version:        0.6.1
Release:        1.corelabels%{?dist}
Summary:        Mission Center fork with named EPP and per-core clocks and identities
License:        GPL-3.0-or-later
URL:            https://github.com/mcdalbu/mission-center
Source0:        %{name}-%{version}.tar.xz

BuildRequires:  rust cargo gcc gcc-c++ meson ninja-build patch
BuildRequires:  blueprint-compiler gettext python3-gobject desktop-file-utils
BuildRequires:  pkgconfig(gtk4) >= 4.16
BuildRequires:  pkgconfig(libadwaita-1) >= 1.6
BuildRequires:  pkgconfig(libdrm) pkgconfig(egl) pkgconfig(gbm) pkgconfig(libudev)
BuildRequires:  pkgconfig(dbus-1) pkgconfig(gl)
Conflicts:      mission-center
Conflicts:      missioncenter

%description
Native fork of Slimbook-Team/mission-center, based on version 0.6.1.
Shows friendly EPP labels and sampled per-logical-CPU frequencies with
P-core/E-core identities on individual CPU graphs. No preload adapter
or power-profile modification is needed by this build.

%prep
%autosetup -n %{name}-%{version}

%build
export CARGO_NET_OFFLINE=true
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_RELEASE_LTO=false
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
export CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_PROFILE_RELEASE_OPT_LEVEL=2
export MC_SOURCE_CACHE="$PWD/build-sources"
meson setup _rpm-build --prefix=%{_prefix} --libdir=%{_libdir} --buildtype=release --wrap-mode=nodownload
ninja -C _rpm-build -j2

%check
rustc --edition=2021 --test src/performance_page/cpu_presentation.rs -o _rpm-build/cpu-presentation-tests
_rpm-build/cpu-presentation-tests
meson test -C _rpm-build --no-rebuild --print-errorlogs

%install
DESTDIR=%{buildroot} meson install -C _rpm-build --no-rebuild
%find_lang missioncenter
# Preserve notices for the Rust crates statically linked into these binaries.
mkdir -p %{buildroot}%{_licensedir}/%{name}/bundled
python3 - <<'PY'
from pathlib import Path
import shutil
out=Path('%{buildroot}%{_licensedir}/%{name}/bundled')
for crate in Path('vendor').iterdir():
    for path in crate.iterdir():
        if path.is_file() and path.name.lower().startswith(('license','copying','notice')):
            dest=out/crate.name
            dest.mkdir(exist_ok=True)
            shutil.copy2(path,dest/path.name)
PY

%files -f missioncenter.lang
%license COPYING
%doc packaging/README.md
%{_bindir}/missioncenter
%{_bindir}/missioncenter-gatherer
%{_datadir}/missioncenter/
%{_datadir}/applications/io.missioncenter.MissionCenter.desktop
%{_datadir}/metainfo/io.missioncenter.MissionCenter.metainfo.xml
%{_datadir}/glib-2.0/schemas/io.missioncenter.MissionCenter.gschema.xml
%{_datadir}/icons/hicolor/*/apps/io.missioncenter.MissionCenter*
%{_licensedir}/%{name}/bundled/

%changelog
* Thu Oct 29 2026 Mission Center fork maintainers - 0.6.1-1.corelabels
- Native EPP names, per-core frequency/type overlays, and offline RPM build.
