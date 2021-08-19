USB Status Lights
=================

This project was designed to let me know at a glance whether nodes in my Raspberry Pi k3s cluster were healthy.

The project comes in two parts, an embedded program that runs on a USB device and a command line application you can use
to control the USB device. This repository provides code for the Adafruit Neo Trinkey, however you could implement the
messaging interface on another device.

Status Lights Neo Trinkey
=========================

Until the `neo_trinkey` library is merged into the main `atsamd-rs` repository, you will need the `atsamd-rs` source
code checked out next to this repository at `../atsamd` and on the `neo-trinkey` branch.

You will also need a recent version of cargo-hf2 installed.

If you have these things you can put your Neo Trinkey into bootloader mode, and run `cargo hf2 --release` in the
`status_lights_neo_trinkey` directory to deploy the code.

Status Lights Neo CLI
=====================

To communicate with the neo trinkey you use the cli tool.

You can install the tool using `cargo install --path status_lights_cli` from the root of this repository. (You can also
just use `cargo run` inside that directory).

To see what devices are installed, you can run `list` which, depending on how many devices you have plugged in, should
produce something like this:

```bash
$ status_lights list
Found 4 devices
/dev/tty.usbmodem145101, Gisleburt Neo Trinkey Status Lights, v0.1.0
/dev/tty.usbmodem145401, Gisleburt Neo Trinkey Status Lights, v0.1.0
/dev/tty.usbmodem1452201, Gisleburt Neo Trinkey Status Lights, v0.1.0
/dev/tty.usbmodem1452101, Gisleburt Neo Trinkey Status Lights, v0.1.0

```
Status light devices connect as a serial device, the first part of the output shows where they are connected. The second
pasrt shows the name (since right now there's only one implementation, these are all the same). The final part shows
what version of the software the USB device is running.

There are then two kinds of lights you can set, background and foreground. These act as layers, if the foreground is
set, the asigned led will shine that color. If it's not, then it will show the background color. If that's not set
either, it will simply be off.

Background colors can be set with:

```bash
$ status_lights background <led> <red> <green> <blue>
```

LEDs are zero indexed and light levels got from 1 to 255. Be warned though, the neo trinkey leds are _very_ bright, so
a low level is recommended.

For example, to set the first led to red, you could use:

```bash
$ status_lights background 0 1 0 0
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145101'
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145401'
```

Note, unless otherwise specified, the cli will always try to set the leds on all connected devices. If you want to only
set the led on one device you can specify the path it's attached to like this:

```bash
$ status_lights background 0 1 0 0 --device /dev/tty.usbmodem145101
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145101'
```

Foreground colors work similarly but can also be set with a length of time in second (**warning:** this is not very
accurate at all and should not be depended on beyond an approximate amount of time). After the given time, the
foreground color will turn off, revealing any previously set background color.

Foreground colors can be set with:

```bash
$ status_lights foreground <led> <red> <green> <blue> [seconds]
```

For example, we can make the led we previously turned red turn to green for 5 seconds (after which it will revert to
red) using the following command.

```bash
$ status_lights foreground 0 0 1 0 5
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145101'
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145401'
```

Again, to only change a specific device, use the `--device` option:

```bash
$ status_lights foreground 0 0 1 0 5 --device /dev/tty.usbmodem145101
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145101'
```

You don't have to set a number of seconds. If seconds is not set (or zero) then the foreground color will not change
back on its own. To go back to the background color, simply set the foreground values to zero. Eg;

```bash
# set led at index 1 to green indefinitely 
$ status_lights foreground 1 0 1 0
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145101'

# restore the led at index 1 to its background color 
$ status_lights foreground 1 0 0 0
Changing device 'Gisleburt Neo Trinkey Status Lights' at '/dev/tty.usbmodem145101'
```

Using the background and foreground commands we can set up some simple status lights using cronjobs. For example, we
could use it to check if there's an internet connection. Consider the following command:

```bash
$ ping 1.1.1.1 -c 1 && status_lights foreground 0 0 1 0 120
```

This would attempt to ping Cloudflares 1.1.1.1 DNS service once, and if successfull it will then set LED 0 to green for
120 seconds(ish).

If we ran this every minute, the light would stay green so long as we have an internet connection. If the internet goes
down for more than two minutes, the light would revent to its background color (we could also set this with a cronjob).

```cron
*/1 * * * * status_lights background 0 1 0 0 120                      # Set the background red
*/1 * * * * ping 1.1.1.1 -c 1 && status_lights foreground 0 0 1 0 120 # Set the foreground green if 1.1.1.1 is reachable
```
