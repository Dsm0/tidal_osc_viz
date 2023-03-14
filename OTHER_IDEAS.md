- pattern the osc messages like a piano roll being recieved
- pattern the osc messages like a piano roll being recieved
- pattern the osc messages like a piano roll being recieved
- pattern the osc messages like a piano roll being recieved

I have an idea for ascii vision of this in my head but I can't be bothered to
write the entire spec out rn in one go.


┌──────────────────────────────────────────────────────────┐
│      Area for viewing individual osc messages            │
│                as they come                              │
│                                                          │
│                                                          │
│                                                          │
│                                                          │
│                                                          │
└──────────────────────────────────────────────────────────┘
------------------------------------------------------------
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│     Area for viewing patterns of messages over time         │
│                                                             │
│                                                             │
│    diff. from current cycle              this is the line of patterns that were triggered
│        ▼                                     ▼  1 cycle behind the current cycle value
│        0         ¼        ½        ¾          1             │
│        ─                                      ─ |           │
│p    "1"| ─►......─►.......─►.......─►........─► |           │
│p    "2"| ─►.─►─►.─►...─►..─►..─►...─►.─►─►...─► |           │
│p    "3"| .                .                   . |           │
│p    "4"| .                .                   . |           │
│p    "5"| .                .                   . |           │
│p    "6"| .                .                   . |           │
│p    "7"| .                .                   . |           │
│p    "8"| .                .                   . |           │
│p    "9"| .                .                   . |           │
│                                                             │
│p "tick"| ─►......─►.......─►.......─►........─►             │  
│                                                             │
└─────────────────────────────────────────────────────────────┘

I imagine the "indiviudal messages" section to be like a collection of meters
of different values (ie "

s    :  :.................laidback_breaks.....................: 
orbit: 0......1......2....##3##....4......5......6......7......8
gain : 0:###################################..................:2
begin: 0:##########...........................................:1
pan  : 0:#######################..............................:1
n    : 0......1......2....##3##....4......5????????????????????????????????????????????????????
ok how do you display an int you have no consistently resaonable bound for?


		")


