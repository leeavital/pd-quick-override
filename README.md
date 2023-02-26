
# `pd-quick-override`

![build status](https://github.com/leeavital/pd-quick-override/actions/workflows/build.yml/badge.svg)


This is a tool to quickly create pagerduty overrides from the command line.



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


## usage: reset-api-key

Clear the API key stored in your local keychain.

```
pd-quick-override reset-api-key
```