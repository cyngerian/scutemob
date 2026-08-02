# Some notes on testing with the current svelte front end:

## Platform
- seed 0 has decks that makes sense
- i tried seed 1 and seed 5 and the commanders didnt fit the cards i was drawing
    - 

## Not enough information:
- commanders dont show card on hover
- events is too sparse and not verbose enough. 
    - there should be 3 versions of events:
        - game level events 
            - player actions 
                - passes priority at end of step
                - draws
                - declares attackers
                - declares blockers
                - plays spell
                - plays land
                - untaps/taps
                - any other major events (begins turn?)
            - card actions:
                - card targets other card
                - card is destroyed/exiled/countered/returned to hand...
                - card is cast
                - card enters battlefield
                - card resolves
                - carb ability triggers
                - any other major card events im missing
            - stack actions 
                - card is on stack at position 0/1/2/3...
                - card is resolved off stack
                - i think a list of prioirty passing makes sense too, but we can discuss
- attacking:
    - not clear which card are attacking which player after attackers declared
    - could be a section under the stack which shows attackers and subsequent blockers
     

## Layout:
- player cards should stay in place on scroll.
- battlefields should be 2x2: tons of empty space on the right of the board
- stack section should 
- target selector should have segments broken up by player
- command zone could just be in player card
- player card shouold be expandable to view cards in hand you know about, but the players hand should be a permananet bar on the bottom like the action bar. 
    - action bar could actually be at the top, under player cards
    - stack under the action bar
- players battlefield should disapear when they die
    - row of battlefields goes away when 2 players are gone, so remaining battlefields render larger


## Playing:
- there needs to be a pass priority button with 2 options:
    - pass until players turn starts
    - pass until card is played or phase end (if this makes sense)
    - this could be fine grained in the future (something i often think about)
        - select player turn and phase and the priority will pass until that phase 
            - ex: Bot-3 end
                - will stop passing when you have priority at bot-3's end step
- tapping 2 mana and then casting a 3 CC card will tap 5 mana total
    - should use mana in pool when casting
- can't play commander
- errors after winning:
    - invariant violations (436): InvariantViolation { check: "stack_consistency", description: "Object ObjectId(403) in Stack zone but not in stack_objects", turn_number: 1 } | InvariantViolation { check: "stack_consistency", description: "Object ObjectId(404) in stack_objects but not in Stack zone", turn_number: 1 } |
- when discarding down from over 7 cards, there is no option to chose which cards to discard. last cards in hard on right get discarded
- played Read the Bones:
    - drew 2 cards, but did not get the ability to scry 2
- diabolic tutor:
    - gave me a button to act when it reolved
    - but it just drew a random card (could have been top card?) instead of letting me select from my library
- bots seem to be tapping mana for no reason, often during upkeeps
    - 2nd game theyre REALLY dumb, playing nothing and discarding all of their creatures 
- i was able to play Boon Sayter with only 1 green mana
    - this was one of the most glaring
        - i noticed it was showing up as an option to cast, which didnt make sense considering the lack of green mana. i clicked it and it actually cast
- Life's Legacy:
    - "The engine refused this play invalid command: spell requires sacrificing a permanent as an additional cost (CR 118.8): SacrificeCreature HTTP 422"
        - ui provides no option to sacrifice a create 
- Galadhrim brigade:
    - no option to pay Squad cost
- Casting cards autotaps mana
    - when i cast Galadhrim brigade, it tapped a sol ring and 2 forests, wasting colorless mana

