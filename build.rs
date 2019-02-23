// **************************************************************************
// Copyright (c) 2015 Roland Ruckerbauer All Rights Reserved.
//
// This file is part of hidapi_rust.
//
// hidapi_rust is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// hidapi_rust is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with hidapi_rust.  If not, see <http://www.gnu.org/licenses/>.
// *************************************************************************

extern crate autotools;
extern crate cc;

use std::process::Command;
use std::env;
use std::path::PathBuf;

#[cfg(windows)]
const GIT: &str = "git.exe";
#[cfg(not(windows))]
const GIT: &str = "git";

fn main() {
	let target = env::var("TARGET").unwrap();

	if target.contains("windows") {
		cc::Build::new()
			.file("etc/hidapi/windows/hid.c")
			.include("etc/hidapi/hidapi")
			.compile("libhidapi.a");
		println!("cargo:rustc-link-lib=setupapi");

	} else if target.contains("darwin") {
		cc::Build::new()
		.file("etc/hidapi/mac/hid.c")
		.include("etc/hidapi/hidapi")
		.compile("libhidapi.a");

		println!("cargo:rustc-link-lib=framework=IOKit");
		println!("cargo:rustc-link-lib=framework=CoreFoundation");

	} else if target.contains("android") {
		enable_android_hack();
		env::set_var("CXX", "arm-linux-androideabi-clang++");
		env::set_var("CC", "arm-linux-androideabi-gcc");
		let libudev = autotools::Config::new("etc/eudev")
			// -s: make symlinks
			// -m: build if it applicable
			// -i: install
			// -v: verbose
			// -f: consider all files obsolete
			.reconf("-smivf")
			.insource(true)
			.host("arm-linux-androideabi")
			.disable_shared()
			.disable("introspection", None)
			.disable("programs", None)
			.disable("hwdb", None)
			.cflag("-D LINE_MAX=2048")
			.cflag("-D RLIMIT_NLIMITS=15")
			.cflag("-D IPTOS_LOWCOST=2")
			.cflag("-std=gnu99")
			.build();

		disable_android_hack();

		let mut config = cc::Build::new();
		config.file("etc/hidapi/linux/hid.c").include("etc/hidapi/hidapi");
		config.compile("libhidapi.a");

		println!("cargo:rustc-link-search=native={}/src/libudev/.libs", libudev.display());
		println!("cargo:rustc-link-lib=static=udev");
	} else if target.contains("linux") {

		let libudev = autotools::Config::new("etc/eudev")
			// -s: make symlinks
			// -m: build if it applicable
			// -i: install
			// -v: verbose
			// -f: consider all files obsolete
			.reconf("-smivf")
			.insource(true)
			.disable_shared()
			.build();

		cc::Build::new()
			.file("etc/hidapi/linux/hid.c")
			.include("etc/hidapi/hidapi")
			.compile("libhidapi.a");

		println!("cargo:rustc-link-search=native={}/src/libudev/.libs", libudev.display());
		println!("cargo:rustc-link-lib=static=udev");
	}
}

fn enable_android_hack() {
	let (start_dir, dst_dir) = get_dirs();
	env::set_current_dir(dst_dir).expect("set current dir to \"etc/eudev\" failed");

	let cmd = Command::new(GIT)
			.args(&["checkout", "83d918449f22720d84a341a05e24b6d109e6d3ae"])
			.status()
			.expect("git checkout failed");
	assert!(cmd.success(), format!("{}", cmd));

	let cmd = Command::new(GIT)
			.args(&["apply", "../libudev.patch"])
			.status()
			.expect("git apply etc/libudev.patch failed");
	assert!(cmd.success(), format!("{}", cmd));

	env::set_current_dir(start_dir).expect("set current dir to \"../..\" failed");
}

fn disable_android_hack() {
	let (start_dir, dst_dir) = get_dirs();
	env::set_current_dir(dst_dir).expect("set current dir to \"etc/eudev\" failed");

	let cmd = Command::new(GIT)
			.args(&["apply", "-R", "../libudev.patch"])
			.status()
			.expect("git revert patch failed");

	assert!(cmd.success(), format!("{}", cmd));

	let cmd = Command::new(GIT)
			.args(&["checkout", "master"])
			.status()
			.expect("git checkout master failed");

	assert!(cmd.success(), format!("{}", cmd));

	env::set_current_dir(start_dir).expect("set current dir to \"../..\" failed");
}

fn get_dirs() -> (PathBuf, PathBuf) {
	let start_dir = env::current_dir().expect("Current dir failed");
	let dst_dir = start_dir.join("etc/eudev");
	(start_dir.to_owned(), dst_dir.to_owned())
}
