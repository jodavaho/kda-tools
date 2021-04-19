This tool will correlate your equipment choices with your KDA spreads.

# What?

I play [hunt showdown](https://www.huntshowdown.com/) a lot. It's very fun. It's also insanely frustrating sometimes. The game has long matches, very frantic, quick battles, and a wide variey of meaningful character specialization and equipment options. It can take dozens of matches to determine if a loadout is worth it, and there are many loadouts, and a match takes an hour ... In short, it *is very hard to get feedback on what equipment loadouts, tactics, or friends are useful*.

For that, I keep a journal of matches, and am writing this tool to output some insights on the data gathered.

To use it, you *will* have to write down match information. But matches last an hour, so that's not much overhead. 

Then, you'll have to use the tools this package provides:

-  `kda-stretch` will stretch out the data in your journal into a stream. This is useful for piping into other programs or your own analysis. Use it like this: `cat journal.txt | kda-stretch` or `<journal.txt kda-stretch`.
-  `kda-corr` the alpha (unstable, unreliable) version of some multi-variate regression that will tell you how well the KDAB spreads you're seeing are explained by your choices in equipment and friends.


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

Then, we can start to explore the data.


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

This means you get on avg 1.67 kills / match with sniper vs 1.00 kills/match without sniper. It  then does a two-sided hypothesis test to see if the rates are equal. 



One way to interpret this is "This doesn't make sense". That's true, it's primitive still.

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
sudo dpkg -i kda-tools_0.1.0_amd64.deb
```

You can use the example above as-is. 

# Todo

- [ ] document match journal format better. 
- [ ] improve match journal to allow :count.
- [ ] provide linter for match journal
- [ ] tool to create / factor matricies in format ammenable to third-party analysis (e.g., R)

# Known issues

- [ ] If you have more variables (long rows of tags / equipment lists) and few games (not very many rows), you'll get a cryptic error about not solving for "W". Add data (play more!), and nag me to implement better methods for inferrence.
- [ ] The equipment tags are *highly correllated*. The way I'm doing statistics right now is *very dumb*.
