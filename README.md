
# `pd-quick-override`

![build status](https://github.com/leeavital/pd-quick-override/actions/workflows/build.yml/badge.svg)


This is a tool to quickly create pagerduty overrides from the command line.


![pd-quick-override-demo](https://user-images.githubusercontent.com/1482532/221438741-23a24f26-f3d2-4d2a-8c02-b6306b9c3b16.gif)


Install with

```
cargo install pd-quick-override
```

You'll also need [FZF](https://github.com/junegunn/fzf) installed on your system.

## usage: create

Create overrides for the current day:

```
pd-quick-override create --at 'today, 4pm-5pm'
pd-quick-override create --at 'today, 4:30pm-5:30pm'
```

Create an override for yourself:

```
pd-quick-override create --me --at 'today, 4:30pm-5:30pm'
```


Specify a timezone (`create` will default to your local timezone)


```
pd-quick-override create --time-zone 'Europe/Paris --at 'today, 4:00pm-5:00pm'
```

Add an override for more than one day:

```
pd-quick-override create --at 'today, 10am - 10/3, 10am'
```



## usage: reset-api-key

Clear the API key stored in your local keychain.

```
pd-quick-override reset-api-key
```
