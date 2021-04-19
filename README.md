This tool will correlate your equipment choices with your KDA spreads.

# What?

I play [hunt showdown](https://www.huntshowdown.com/) a lot. It's very fun. It's also insanely frustrating sometimes. The game has long matches, very frantic, quick battles, and a wide variey of meaningful character specialization and equipment options. It can take dozens of matches to determine if a loadout is worth it, and there are many loadouts, and a match takes an hour ... In short, it *is very hard to get feedback on what equipment loadouts, tactics, or friends are useful*.

For that, I keep a journal of matches, and am writing this tool to output some insights on the data gathered.

To use it, you *will* have to write down match information. But matches last an hour, so that's not much overhead. 

Then, you'll have to use the tools this package provides:

-  `kda-summary` will summarize your K, D, and A values (and the usual KDA metric) over the entire journal.
-  `kda-compare` the alpha (unstable, unreliable) version of some multi-variate hyothesis tests that will tell you if you're doing significantly differently with different loadouts.


# Basics

Keep a match journal like this (fyi this is [key value count format](https://github.com/jodavaho/kvc) ).

```
<date> [<items or friends initials>] [K|D|A|B]
```

For example, my Hunt diary looks a little like:
```
2021-03-12 BAR+Scope pistol K K B alone
2021-03-12 BAR+Scope pistol K D D jb
2021-03-12 Short-Rifle Short-Shotgun K D jb
2021-03-12 BAR+Scope pistol D jp+jb
2021-03-13 BAR+Scope pistol jp D
2021-03-13 BAR+Scope pistol jp B D D A A
2021-03-13 Shotgun pistol jp D
2021-03-13 BAR+Scope pistol jp K
2021-03-14 Short-Rifle akimbo  alone
2021-03-17 LAR Sil pistol  alone
2021-03-17 pistol-stock akimbo  alone
2021-03-17 Short-Shotgun pistol-stock  alone
```

you can use whatever you want to denote loadouts or friends ... it'll just run multi-variate regression on all of them with the important parts: `K D A or B`, for example...
Contents of journal.txt:

```
K K Sniper
K D Shotgun
K K JP Sniper
K D B Shotgun JB
K D B Sniper JB
```

is 5 matches:

1. two kills with a sniper loadout
2. a kill a death with a shotgun loadout
3. two kills with a sniper loadout and team-mate "JP"
4. a kill, a death, a bounty with Shotguns and team-mate "JB"
5. Same, but with Sniper loadout

# KDA-Summary

Let's see the summary over time:

```bash
$ <journal.txt kda-summary
    n       Date   K   D   A   B   KDA    sK    sD    sA    sB  mKDA    mK    mD    mA    mB 
    1          1   2   0   0   0  2.00     2     0     0     0  2.00  2.00  0.00  0.00  0.00 
    2          2   1   1   0   0  1.00     3     1     0     0  3.00  1.50  0.50  0.00  0.00 
    3          3   2   0   0   0  2.00     5     1     0     0  5.00  1.67  0.33  0.00  0.00 
    4          4   1   1   0   1  1.00     6     2     0     1  3.00  1.50  0.50  0.00  0.25 
    5          5   1   1   0   1  1.00     7     3     0     2  2.33  1.40  0.60  0.00  0.40 
```

Not bad. Notice, `kda-summary` *requires* the use of tags `K` for kills, `D` for death, `B` for bounties, and `A` for assist. It outputs your per-match stats, the KDA value of `(K+A)/D`, the sum of the K, D, A and B, and the mean (avg / match) of KDA, K, D, A, and B.

# KDA-Compare

With 
`kda-compare` 
the semantics of 'K' vs 'k' vs "kill" is irrelevant. We explore the data by asking it to analyze variables by name. For example, in the  data above, to see kills "K" per match with Sniper and without, you form the "experiment" denoted as "K:Sniper" and ask `kda-compare` to run that experiment by `kda-compare -c "K : Sniper"`

You can run many experiments seperated by 'vs' (this will change), against many output varaibles ... All are valid:

- `kda-compare -c "K D : Sniper vs Shotgun"` to see which is better
- `kda-compare -c "D : K"` to see if you die more when you kill stuff or not
- `kda-compare -c "Sniper: JB"` to see if you play sniper more or less when you're with JB

and so on ... each "tag" (item on a line in a journal) is a valid input or output depending on your determination of experiments.

Heres one:
```bash
$ <journal.txt kda-compare -c "K D : Sniper vs Shotgun"
Processed. Read: 5 rows and 7 variables
K Sniper Shotgun D JP JB B 
Debug: processing: K: Sniper vs Shotgun
K:( Sniper ) 5.00/3 = 1.67  vs 2.00/2 = 1.00 Rates are same with p=0.373
K:( Shotgun ) 2.00/2 = 1.00  vs 5.00/3 = 1.67 Rates are same with p=0.373
```

We note that the rates of kills with shotgun exactly equal the rates of kills w/ *not sniper*, so this test results are the same. 

## Use

**Warning, this functionality will change rapidly prior to 1.0 release**

```bash
$ kda-compare -h
USAGE:
    kda-compare [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
        --kda        Include the extra output KDA = (K+A)/D. You'll need to have K, D, and A entries in your log or this
                     will fail loudly.
    -V, --version    Prints version information

OPTIONS:
    -c, --command <COMMAND>    Command a comparison like this: 'K (: [<item>] vs [<item>] )' e.g., 'K: pistol vs
                               shotgun' to compare kills with shotguns vs pistols. or 'K:pistol' to check pistols vs non-pistols
```


## Use cases

To check the efficacy of the "Sniper" loadout
```
< journal.txt kda-compare -c "K: Sniper"
```

At this time (v0.5.0) it produces:

```bash
Processed. Read: 5 rows and 7 variables

K Sniper Shotgun D JP B JB
Debug: processing: K:Sniper
K:( Sniper ) 5.00/3 = 1.67  vs 2.00/2 = 1.00 Rates are same with p=0.373
```

This means you get on avg 1.67 kills / match with sniper vs 1.00 kills/match without sniper. It  then does a two-sided hypothesis test to see if the rates are equal. If p is low (say, lower than .1) you can consider them different. More data makes the results stronger. Strength-of-predictor will be added soon.

One way to interpret this is "This doesn't make sense". That's true, it's primitive still, and mostly a toy for my own use.

# Get for debian / WSL

For now ...

```
wget josh.vanderhook.info/kda-tools_0.5.0_amd64.deb 
md5sum  kda-tools_0.1.0_amd64.deb
```
Output had better be:
```
6350920a358a6e1c05579034bac85911  /home/hook/ws/hunt/dev/target/debian/kda-tools_0.5.0_amd64.deb
```

If so, then, 

```
sudo dpkg -i kda-tools_0.5.0_amd64.deb
```

You can use the example above as-is. 

# Todo

- [x] document match journal format better. see: github.com/jodavaho/kvc.git
- [x] improve match journal to allow :count. (see kvc again)
- [ ] provide linter for match journal
- [ ] tool to create / factor matricies in format ammenable to third-party analysis (e.g., R)
- [ ] Perform power tests / experiment design 
- [ ] remove '-c' as mandatory switch ... obsolete when baseline '\_' was removed
- [ ] Provide a library version in C, 
  - [ ] C++, 
  - [ ] Rust

# Known issues

- If you have an item you use every game, then you have insufficient data. for every test of the form `k:A` there *must* be at least one match *without* A occuring. It's ok if there's no kills (`k`).
