These tools help you run [Alpha/Beta](https://en.wikipedia.org/wiki/A/B_testing) tests on your equipment choices for multiplayer competitive games. In particular, it is good for checking the [KDA](https://slangit.com/meaning/kda)

# What?

I play [hunt showdown](https://www.huntshowdown.com/) a lot. It's very fun. It's also insanely frustrating sometimes. The game has long matches, very frantic, quick battles, and a wide variey of meaningful character specialization and equipment options. It can take dozens of matches to determine if a loadout is worth it, and there are many loadouts, and a match takes an hour ... In short, it *is very hard to get feedback on what equipment loadouts, tactics, or friends are useful*.

For that, I keep a journal of matches, and am writing this tool to output some insights on the data gathered.

To use it, you *will* have to write down match information. But matches last an hour, so that's not much overhead. 

Then, you'll have to use the tools this package provides:

-  `kda-summary` will summarize your K, D, and A values (and the usual KDA metric) over the entire journal.
-  `kda-compare` will run will look at the whole dataset and tell you if you're doing significantly differently with different loadouts.
-  `kda-explore` will allow *you* to look at the conditional distributions *of anything* with and without *anything else*.


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

Contents of examples/journal.txt:

```
2021-01-03 K K Sniper
2021-01-03 K D Shotgun
2021-01-04 K K JP Sniper
2021-01-04 K D B Shotgun JB
2021-01-04 K D B Sniper JB
```

is 5 matches:

1. two kills with a sniper loadout
2. a kill a death with a shotgun loadout
3. two kills with a sniper loadout and team-mate "JP"
4. a kill, a death, a bounty with Shotguns and team-mate "JB"
5. Same, but with Sniper loadout

# Automated: KDA-Summary

Let's see the summary over time:

```bash
$ <journal.txt kda-summary
    n       Date   K   D   A   B   KDA    sK    sD    sA    sB  mKDA    mK    mD    mA    mB
    1 2021-01-03   2   0   0   0  2.00     2     0     0     0  2.00  2.00  0.00  0.00  0.00
    2 2021-01-03   1   1   0   0  1.00     3     1     0     0  3.00  1.50  0.50  0.00  0.00
    3 2021-01-04   2   0   0   0  2.00     5     1     0     0  5.00  1.67  0.33  0.00  0.00
    4 2021-01-04   1   1   0   1  1.00     6     2     0     1  3.00  1.50  0.50  0.00  0.25
    5 2021-01-04   1   1   0   1  1.00     7     3     0     2  2.33  1.40  0.60  0.00  0.40
```

Not bad. Notice, `kda-summary` *requires* the use of tags `K` for kills, `D` for death, `B` for bounties, and `A` for assist. It outputs your per-match stats, the KDA value of `(K+A)/D`, the sum of the K, D, A and B, and the mean (avg / match) of KDA, K, D, A, and B.

Note the `Date` field. If you put dates of the form `YYYY-MM-DD` somewhere per line in the journal, it will populate that field. See example above or [kvc](github.com/jodavaho/kvc.git). File bug reports *there* if you don't like how the dates are required to be formatted.

# Semi-Automated: KDA-compare

So, what's the better loadout or partner?  KDA-compare does some testing for you.

```bash
$ <journal.txt kda-compare
Processed. Read: 5 rows and 8 variables

[====================================================] 100.00 % 2696.14/s 
met    grp      n/d      val   N     n/d      ~val  M     p
kda    Sniper   5/1      5.00   3    2/2      1.00   2    0.06
kda    JP       2/0      inf    1    5/3      1.67   4    0.49
kda    Shotgun  2/2      1.00   2    5/1      5.00   3    0.77
kda    JB       2/2      1.00   2    5/1      5.00   3    0.78
b/d    Sniper   1/1      1.00   3    1/2      0.50   2    0.24
b/d    Shotgun  1/2      0.50   2    1/1      1.00   3    0.69
b/d    JB       2/2      1.00   2    0/1      0.00   3    NaN
b/d    JP       0/0      NaN    1    2/3      0.67   4    NaN
```

Let's look. The first row is `met grp ...`
These are 
- the metric name (e.g., kda or bounties / death b/d)
-  item group (grp)
-  value counts (n)
-  deaths (d)
- the value of the metric 'val' *with* the grp
- number of matches where 'grp' was used (N)
-  value counts *without* the grp (n)
-  death counts *without* the grp (d)
- the value of the metric 'val' *without* the grp
-  number of matches *without* the grp (M)
-  and the probability that we'd randomly see that 'val' given the distribution of the metric without the grp. 

You can see a pvp metric (kda or (kills + assists)/deaths ), and pve metric (bounties/death), for each item.

That last one, p, is usually called a p-value, and if it's low, you have a
signfiicantly *better* set of rounds with the grp than without it. 

In the data above, it appears that rounds where I use the `Sniper` weapon are significantly better than rounds where I don't, given `p` is small. 

Note, there are some `NaN`'s because when playing with JP I got no bounties or
deaths (0/0), which is not a meaningful result to compare against. Less
obviously, when playing with JB, I *also* cannot get a p-value, since in the
rounds that I did *not* play with JB , I *never got a bounty*. This means
there's no meaningful representation for the *baseline* (without JB) case. 

OK, so what? well, draw your own conclusions and try to mix up your loadouts.
If you only play snipers wtih your friend JP, and only play shotguns with your
friend JB, they will be highly correlated and it may be hard to see if JB or
Shotgun makes the most difference. 

There is an option `-i` to ignore certain items. This is useful if you want to see the weapons only. 

let's ignore my wingmen JP and JB to check just weapons.

```bash
$ <journal.txt kda-compare -i JP JB
Processed. Read: 5 rows and 8 variables

[====================================================] 100.00 % 3450.51/s 
met    grp      n/d      val   N     n/d      ~val  M     p
kda    Sniper   5/1      5.00   3    2/2      1.00   2    0.08
kda    Shotgun  2/2      1.00   2    5/1      5.00   3    0.77
b/d    Sniper   1/1      1.00   3    1/2      0.50   2    0.24
b/d    Shotgun  1/2      0.50   2    1/1      1.00   3    0.69
```

Now we see a slight decrease in Sniper for kda, but no change in b/d. 

You can also test pairings (group-size = 2)


```
$ <journal.txt kda-compare --group-size 2
Processed. Read: 5 rows and 8 variables

[==================================================] 100.00 % 8175.42/s 
met    grp         n/d      val   N     n/d      ~val  M     p
kda    JP+Sniper   2/0      inf    1    5/3      1.67   4    0.47
kda    JB+Sniper   1/1      1.00   1    6/2      3.00   4    0.78
kda    JB+Shotgun  1/1      1.00   1    6/2      3.00   4    0.79
b/d    JB+Shotgun  1/1      1.00   1    1/2      0.50   4    0.62
b/d    JB+Sniper   1/1      1.00   1    1/2      0.50   4    0.64
b/d    JP+Sniper   0/0      NaN    1    2/3      0.67   4    NaN
```

Now we see that there isn't really a difference across friend / weapon
pairings. Note the `inf` result. You may think that infinites are not possible
to analyze. Well, since we're using bootstrap methods (see
[poisson-rate-test](github.com/jodavaho/poisson-rate-test)), you can actually
get meaningful probabilities, and therefore meaningful p-values.

To confirm that friends are a nuisance variable (no statistically significant difference), try ignoring weapons:

```
$ <journal.txt kda-compare -i Sniper Shotgun
Processed. Read: 5 rows and 8 variables

[====================================================] 100.00 % 3948.08/s 
met    grp n/d      val   N     n/d      ~val  M     p
kda    JP  2/0      inf    1    5/3      1.67   4    0.49
kda    JB  2/2      1.00   2    5/1      5.00   3    0.77
b/d    JB  2/2      1.00   2    0/1      0.00   3    NaN
b/d    JP  0/0      NaN    1    2/3      0.67   4    NaN
```

here we definitely see that the choice of wingman has much less effect than the
choice of weapon (compare values from two examples ago)

## use
```bash
$ kda-compare -h
It *expects* input in kvc format (one match per line), and processs the variables K, D, and A, as a function of *all
other* variables present. It ignores kvc keywords / fields (like dates), but you'll have to specify other things to
ignore manually.


USAGE:
    kda-compare [FLAGS] [OPTIONS]

FLAGS:
    -f               Speed up computation by doing a fewer number of iterations. Helpful for quick looks but the
                     ordering of some sets may change across multiple invocations
    -h, --help       Prints help information
    -n               Display notes about particular test cases in the output
    -V, --version    Prints version information

OPTIONS:
    -g, --group-size <group_size>    Instead of individual items (group_size==1), rank by enumerated groupings that
                                     appear in data of a given size. [default: 1]  [possible values: 1, 2, 3, 4]
    -i <ignore>...                   List of fields to ignore (if they appear in data). You can ignoring fields A B and
                                     C as '-i A,B,C' or '-i A -i B -i C' but not '-i A B C' or '-i A, B, C'. That's
                                     because of shell magic, not becuase of the way it was implemented
    -o <out_format>                  Output format which can be one of Vnlog or Whitespace-,  Tab-, or Comma-seperated.
                                     [default: wsv]  [possible values: wsv, tsv, csv, vnl]
```

# Manual: KDA-Explore 

To really dig in, you can evaluate the conditional distribtuions of any variable conditioned on the occurance of any other variable. This is the role of `kda-explore`.

What I mean is that `kda-explore` the semantics of 'K' vs 'k' vs "kill" is
irrelevant. We explore the data by asking it to analyze variables by name. For
example, in the data above, to see kills "K" per match with Sniper and without,
you form the "experiment" denoted as "K:Sniper" and ask `kda-explore` to run
that experiment by `kda-explore "K : Sniper"`. 

```bash
$ < journal.txt kda-explore K:Sniper
Processed. Read: 5 rows and 8 variables
Varibables found: Date Sniper K D Shotgun JP B JB 
Debug: processing: K:Sniper
met    grp     n     M     rate  ~n    ~M    ~rate p     notes
K      Sniper  5     3     1.67  2     2     1.00  0.37
```

This means the probability of a kill given you have a sniper is higher, but not so high that it might not be much different over time (p .37). We're only checking kills here. What about deaths?

```bash
$ < journal.txt kda-explore D:Sniper
Processed. Read: 5 rows and 8 variables
Varibables found:
Date Sniper K Shotgun D JP B JB 
Debug: processing: D:Sniper
met    grp     n     M     rate  ~n    ~M    ~rate p     notes
D      Sniper  1     3     0.33  2     2     1.00  0.19
```

You can run many experiments seperated by 'vs' (this may change), against many output varaibles ... All are valid:

- `kda-explore "K D : Sniper vs Shotgun"` to see kills and deaths compared to sniper and shotguns
- `kda-explore "D : K"` to see if you die more when you kill stuff or not
- `kda-explore "Sniper: JB"` to see if you play sniper more or less when you're with JB
- `kda-explore K:all` to see kill spreads and sorted rate comparisons for all variables

and so on ... each "tag" (item on a line in a journal) is a valid input or output depending on your determination of experiments.

## Use

```bash
$ kda-explore -h
USAGE:
    kda-explore [OPTIONS] <command>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o <out_format>        Output format which can be one of Vnlog or Whitespace-,  Tab-, or Comma-seperated. [default:
                           wsv]  [possible values: wsv, tsv, csv, vnl]

ARGS:
    <command>    The A/B comparison to run, of the form '<some variables : <other variables>'. e.g., 'K: pistol'
                 will check kills with and wtihout pitols [default: K D A : all]
```


One way to interpret this is "This doesn't make sense". That's true, it's primitive still, and mostly a toy for my own use.

# Get for debian / WSL

For now, just grab one of the test debs in releases/

Then,

```
sudo dpkg -i kda-tools_0.5.0_amd64.deb
```

You can use the example above as-is. 

# Todo

- [x] document match journal format better. see: github.com/jodavaho/kvc.git
- [x] improve match journal to allow :count. (see kvc again)
- [ ] provide linter for match journal
- [x] tool to create / factor matricies in format ammenable to third-party analysis (e.g., R)
- [ ] Perform power tests / experiment design 
- [x] remove '-c' as mandatory switch ... obsolete when baseline '\_' was removed
- [ ] Provide a library version in C, 
  - [ ] C++, 
  - [x] Rust

# Known issues

- If you have an item you use every game, then you have insufficient data. for every test of the form `k:A` there *must* be at least one match *without* A occuring. It's ok if there's no kills (`k`).
