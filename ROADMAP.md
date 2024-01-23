# Roadmap

This is the feature roadmap for [beancount.bot](https://beancount.bot).

It's sorted in priority order, so I'm either currently working on the top item
on the list or planning to start work on it soon. If you have a feature
suggestion, let me know! You can submit feature requests via a GitHub issue, or
by sending me an email at
<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>.

## Meal Recommendations

Use the LLM to provide suggested meals. We'll create a prompt which takes
several pieces of information into account:

- user's past meals
- the user's time of day
- how many calories the user has eaten / how much of their calorie goal remains
- which ingredients the user has at home
- how much time the user wishes to spend preparing food
- whether the user intends to make food or order food

## Consumption Metrics

When a calorie goal is defined, show a progress bar visualization.

## Meal Pacing

Help users understand if they are ahead or behind pace. This should incorporate
the users' plan for the day, for example, planning on eating a large dinner.

## Exercise Tracking

Use a similar LLM interface to track exercise, which would add to calorie
budgets.


## Macro Goals

Allow the input of macro goals (grams of protein, fat, and carbs).

# Completed Items

Looking back fondly at the road behind us.

## Automatic Calorie Balancing

Apply deficit or excess calories after each day to the following days, ensuring
a continuous coercion towards the net calorie goal.

### Implementation Details

New Pages:

- [x] calorie balancing page, which provides introspection into where the current
  calorie goal came from
- [x] list of "resets" -- dates at which the auto-balancing has been reset

Updates to the home page:

- [x] add some ugly text indicating that the calorie goal came from auto-balancing
- [x] put a hyperlink in this text, which points to the calorie balancing page

#### Caching

I can't justify building out caching. Meals in the context of calorie balancing
is just a tuple of calorie and datetime. It's 96 bits each. 1kb will hold over
8,000 meals. 20 meals per day is way up on the top end of the bell curve for my
own usage (obviously `meal` is a count of food items entered, not necessarily
meals, so it's going to be much more than 3. My median is 9.5).

Anyway, this means that I'd need to use the app for over 2 years before the data
dependency of my daily meal count rises above 1kb. Obviously, if we get a decent
number of users, the `meal` table will get quite big, but we can create a
[covering
index](https://www.postgresql.org/docs/current/indexes-index-only-scans.html) to
ensure that the `created_at` and `calories` columns are included in the
index on `user_id`. This would create locality and allow us to satisfy probably
several years of meal data in one or two PostgreSQL pages.

All together, there's no point even thinking about how I'd implement caching. I
need to stop myself.

_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_
_I will not implement unnecessarily complicated caching_

Hm nice, the chalk-board punishment is much easier in vim.
