# reflector-rs

reflector-rs is a Rust implementation of the [Reflector][reflector]
script for retrieving up-to-date Arch Linux mirror data from the [Mirror
Status][mirror-status] web interface.

This project aims to provide as close to a 1:1 implementation with the original
`reflector` script while minimizing the number of dependencies.

[reflector]: https://xyne.dev/projects/reflector/
[mirror-status]: https://www.archlinux.org/mirrors/status/

## Installation
Use your preferred AUR helper to directly install the package:

```bash
# With yay
yay -S reflector-rs

# With paru
paru -S reflector-rs
```

Alternatively, the binary can be installed via cargo.

```rust
cargo install arch-reflector
```

>[!WARNING]
>This will only install the binary and put it on your local user's PATH, this
>not install it system wide and does not come with the packaged man pages or
>systemd units.

## Usage
```
Retrieve and filter a list of the latest Arch Linux mirrors.

Usage: reflector [OPTIONS]

Options:
      --url <URL>
          The URL from which to retrieve the mirror data in JSON format. If different from the default, it must follow the same format

          [default: https://archlinux.org/mirrors/status/json/]

      --list-countries
          Display a table of the distribution of servers by country

  -v, --verbose...
          Increase logging verbosity

  -q, --quiet...
          Decrease logging verbosity

      --connection-timeout <n>
          The number of seconds to wait before a connection times out

          [default: 5]

      --download-timeout <n>
          The number of seconds to wait before a download times out

          [default: 5]

      --cache-timeout <n>
          The cache timeout in seconds for the data retrieved from the Arch Linux Mirror Status API

          [default: 300]

      --save <filepath>
          Save the mirrorlist to the given file path

      --sort <SORT>
          Sort the mirrorlist by the given field

          Possible values:
          - age:     last server synchronization
          - rate:    download rate Rate,
          - country: country name, either alphabetically or in the order given by the --country option
          - score:   MirrorStatus score
          - delay:   MirrorStatus delay

      --threads <THREADS>
          Use n threads for rating mirrors. This option will speed up the rating step but the results will be inaccurate if the local bandwidth is saturated at any point during the operation. If rating takes too long without this option then you should probably apply more filters to reduce the number of rated servers before using this option

          [default: 0]

      --info
          Print mirror information instead of a mirror list. Filter options apply

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

filters

The following filters are inclusive, i.e. the returned list will only contain mirrors for which all of the given conditions are met.
:
  -a, --age <n>
          Only return mirrors that have synchronized in the last n hours. n may be an integer or a decimal number

      --delay <n>
          Only return mirrors with a reported sync delay of n hours or less, where n is a float. For example. to limit the results to mirrors with a reported delay of 15 minutes or less, pass 0.25

  -c, --country <country name or code>
          Restrict mirrors to selected countries. Countries may be given by name or country code, or a mix of both. The case is ignored. Multiple countries be selected using commas (e.g. --country France,Germany) or by passing this option multiple times (e.g.  -c fr -c de). Use "--list-countries" to display a table of available countries along with their country codes. When sorting by country, this option may also be used to sort by a preferred order instead of alphabetically. For example, to select mirrors from Sweden, Norway, Denmark and Finland, in that order, use the options "--country se,no,dk,fi --sort country". To set a preferred country sort order without filtering any countries.  this option also recognizes the glob pattern "*", which will match any country. For example, to ensure that any mirrors from Sweden are at the top of the list and any mirrors from Denmark are at the bottom, with any other countries in between, use "--country 'se,*,dk' --sort country". It is however important to note that when "*" is given along with other filter criteria, there is no guarantee that certain countries will be included in the results. For example, with the options "--country 'se,*,dk' --sort country --latest 10", the latest 10 mirrors may all be from the United States. When the glob pattern is present, it only ensures that if certain countries are included in the results, they will be sorted in the requested order

  -f, --fastest <n>
          Return the n fastest mirrors that meet the other criteria. Do not use this option without other filtering options

  -i, --include <regex>
          Include servers that match <regex>, where <regex> is a Rust regular express

  -e, --exclude <regex>
          Exclude servers that match <regex>, where <regex> is a Rust regular expression

  -l, --latest <n>
          Limit the list to the n most recently synchronized servers

      --score <n>
          Limit the list to the n servers with the highest score

  -n, --number <n>
          Return at most n mirrors

  -p, --protocol <protocol>
          Match one of the given protocols, e.g. "https" or "ftp". Multiple protocols may be selected using commas (e.g. "https,http") or by passing this option multiple times

      --completion-percent <[0-100]>
          Set the minimum completion percent for the returned mirrors. Check the mirror status webpage for the meaning of this parameter

          [default: 100]

      --isos
          Only return mirrors that host ISOs

      --ipv4
          Only return mirrors that support IPv4

      --ipv6
          Only return mirrors that support IPv6
```

### Examples
Print the latest mirrorlist to STDOUT:

```bash
reflector
```

Sort the five most recently synchronized mirrors by download speed and overwrite
the local mirrorlist:

```bash
reflector --latest 5 --sort rate --save /etc/pacman.d/mirrorlist
```

Select the 200 most recently synchronized HTTP or HTTPS mirrors, sort them by
download speed, and overwrite the file /etc/pacman.d/mirrorlist:

```bash
reflector --latest 200 --protocol http,https --sort rate --save /etc/pacman.d/mirrorlist
```

Select the HTTPS mirrors synchronized within the last 12 hours and located in
either France or Germany, sort them by download speed, and overwrite the file
/etc/pacman.d/mirrorlist with the results:

```bash
reflector --country France,Germany --age 12 --protocol https --sort rate --save /etc/pacman.d/mirrorlist
```

## systemd Integration
Reflector includes systemd service and timer units that can be used to
automatically update Pacman’s mirrorlist. To use the timer, edit the
configuration file at **/etc/xdg/reflector/reflector.conf** and then enable the
timer with

```
systemctl enable reflector.timer
systemctl start reflector.timer
```

Check that status with `systemctl list-timers`. To update the mirrorlist
immediately instead of waiting for the scheduled operation, run `systemctl start
reflector.service`.

## Minimum Supported Rust Version
All crates under this repository use a MSRV matched with the latest release in
Arch's repositories (currently 1.99).
