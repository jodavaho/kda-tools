# What?

I play [hunt showdown](https://www.huntshowdown.com/) a lot. It's very fun. It's also insanely frustrating sometimes. The game has long matches, very frantic, quick battles, and a wide variey of meaningful character specialization and equipment options. It can take dozens of matches to determine if a loadout is worth it, and there are many loadouts, and a match takes an hour ... In short, it *is very hard to get feedback on what equipment loadouts, tactics, or friends are useful*.

For that, I keep a journal of matches, and am writing this tool to output some insights on the data gathered.

To use it, you *will* have to write down match information. But matches last an hour, so that's not much overhead. Then, you'll have to use the two tools this provides:

-  `kda-stretch` will stretch out the data in your journal into a stream. This is useful for piping into other programs or your own analysis. Use it like this: `cat journal.txt | kda-stretch` or `<journal.txt kda-stretch`.
-  `kda-corr` the alpha (unstable, unreliable) version of some multi-variate regression that will tell you how well the KDAB spreads you're seeing are explained by your choices in equipment and friends.


# Basics

Keep a match journal like this:

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
Tues Skirmish w/Alice A A D  B
Wed SniperBuild w/John K K K 
Thurs Skirmish w/John D A
```

is two assists, a death and a bounty on Tuesday w/ alice;
a hat trick of kills w/ John on wed, and a death and assist the following day. 

Then, we can try out this data:

```
< journal.txt | kda-stretch | kda-corr
```

The output is something like:

```
  ┌                                  ┐
  │          0   0.558746          0 │
  │          0 0.20674208          0 │
  │          1 0.07981821          1 │
  │          0  4.5680423          0 │
  │          0 -1.6212965          0 │
  └                                  ┘


         Weight:   K-D   A-D     B
           Time:  0.00  0.00  1.00
       Skirmish:  0.00  0.00  0.56
        w/Alice:  0.21  0.08  4.57
    SniperBuild: -1.62  0.00  0.00
         w/John:  1.00  0.00  0.00
```

The top row says "You're getting more bounties as time goes on". The rest of the rows tell you how well the observed K-D, A-D spreads or B counts are explained by the factor in each row.

One way to interpret this is "This doesn't make sense". That's true, it's primitive still.

But, high positive numbers for w/Alice and bounty counts means your best bet for bounties is in playing with Alice. That's obvious from the journal. It also says the sniper build isn't very helpful, but skirmish isn't either. It says the main thing that matters when looking at K-D is playing w/ John. 

# Get for debian / WSL

```
wget josh.vanderhook.info/kda-tools_0.1.0_amd64.deb 
md5sum  kda-tools_0.1.0_amd64.deb
```
Output had better be:
```
2a36501e7f234034bd778d8ccb8cf736  target/debian/kda-tools_0.1.0_amd64.deb
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
