# Roadmap

This is the feature roadmap for [beancount.bot](https://beancount.bot).

It's sorted in priority order, so I'm either currently working on the top item
on the list or planning to start work on it soon. If you have a feature
suggestion, let me know! You can submit feature requests via a GitHub issue, or
by sending me an email at
<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>.

## Automatic Calorie Balancing

Apply deficit or excess calories after each day to the following days, ensuring
a continuous coercion towards the net calorie goal.

### Implementation Details

We should work from the inside out. Ultimately, a consideration is that this
system naively might aggregate over _all_ user meals, not just the current
day, so we want to think ahead in terms of performance. At least, we want
a core layer which will work if caching is introduced, even if we don't do any
caching yet.

Another consideration is just how, exactly, we want this system to work, and
which configuration options we want to provide. Should it look back infinitely
into the past? Should diffs fall off after a period of time? Should that time
period be adjustable? Should some diff threshold exist where, if exceeded, we
will ignore?

First, it's safe to say that it would not be reasonable to ask the app to, for
example, spread a calorie excess over a very long period of time like a year.
I'm sorry, but if I binged on a cheesecake, amortizing those calories over the
next decade doesn't help me reach my weight loss goals, ultimately! So, we can
set a maximum window of, say, 30 days, and a default window of 7 days. 5 feels
like a good default because any excess eating on the weekend is then accounted
for during the following week. Plus, I feel that 500-1000 calories is a pretty
typical weekend excess for me, so a resultant 100-200 cut-back in the following
days seems reasonable and achievable. I think there are more bells and whistles
that are possible here, like spreading especially large meals (think:
thanksgiving dinner) over a longer period of time. Another future direction for
this feature is setting upper and lower thresholds for rollover, where if you
eat more than 1,000 calories in excess or deficit compared to your goal, maybe
we don't roll that over. Still, I think this is not as important as basic
rollover.

Taking a step back, it also occurs to me that this feature puts more emphasis on
the need for users to track _all_ calories. The whole system gets messed up if
users don't track every calorie, since we end up rolling over calorie deficits
and increasing calorie goals. What can we do about that?

(planning on this feature is a work in progress)

## Consumption Metrics

When a calorie goal is defined, show a progress bar visualization.

## Meal Recommendations

Use the LLM to provide suggested meals. We'll create a prompt which takes
several pieces of information into account:

- user's past meals
- the user's time of day
- how many calories the user has eaten / how much of their calorie goal remains
- which ingredients the user has at home
- how much time the user wishes to spend preparing food
- whether the user intends to make food or order food

## Meal Pacing

Help users understand if they are ahead or behind pace. This should incorporate
the users' plan for the day, for example, planning on eating a large dinner.

## Exercise Tracking

Use a similar LLM interface to track exercise, which would add to calorie
budgets.


## Macro Goals

Allow the input of macro goals (grams of protein, fat, and carbs).

