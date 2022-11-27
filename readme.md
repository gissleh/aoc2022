# AOC 2022

...and some stuff from previous years. There are feature flags to keep the
build fast when just one year is being considered.

Scripts:
* `./dl-input.sh` – Download all inputs from this year.
* `./dl-old-input.sh 2015 1` – Download specific input file.

The scripts reads the file `cookie.env`:

```
AOC_SESSION=536...
```