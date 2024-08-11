# Roadmap

This is the feature roadmap for [beancount.bot](https://beancount.bot).

It's sorted in priority order, so I'm either currently working on the top item
on the list or planning to start work on it soon. If you have a feature
suggestion, let me know! You can submit feature requests via a GitHub issue, or
by sending me an email at
<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>.

# Upcoming Features

## Meals

Group multiple food items into a meal. Plan meals interactively before you sit
down to eat. Add meals to the current day to track your calories.

At first, I was thinking about strictly deduping food items per-user by
maintaining a unique key on `(user_id, food_name)`. This would have provided the
advantage that we would understand the history of when and in what quantity a
given food item is eaten, which could provide some interesting insights. We can
also understand food items that are shared across saved meals, and all of the
instances that a set of food items (i.e, saved meals) were eaten. It seems like
some degree of deduping provides nice properties.

However, strict deduping presents a challenge for basic food item insertion.
Consider if you type a simple query like `120 calories of protein powder`.
ChatGPT may return slightly different macros the second time that you type this,
but the name would clash with the first. What do we do in this scenario? We
can't overwrite the macros of the first protein powder; this would surprisingly
change past meals and modify your calorie balancing results. I think that strict
deduping is not the right solution, because it is too rigid and will introduce
too much complexity.

Instead, I think that we should add natural deduping to the app;

- separate `food` and `food_eaten_event` into different tables
- point saved meals and their eaten events back to the same row in `food`. This
  can be part of the operation for adding a saved meal to the current day
- modify the behavior of the "add meal to today" button to insert into
  `food_eaten_event` a pointer back to the same `food` item instead of cloning
- in the future, potentially search for similar `food` in addition to querying
  ChatGPT when new food items are added, and present an option to use previously
  entered food

If this is all designed well, deduping should naturally emerge, while we also
keep the friction low for adding items if duplicates do happen to be introduced.
Plus, as we add the nice features which take advantage of soft deduping, some
users may be incentivized to proactively dedupe as they also build up libraries
of data that they trust more than LLM responses.

## Calorie Progress Bar

Show a progress bar instead of, "You have 1000 calories left to eat today." for
users that have a caloric intake goal.

# Completed Items

Looking back fondly at the road behind us.

## Blog

It's content marketing time, baby (no posts yet).

## Marketing Efforts

- [x] add some hero text to the home page
- [x] get rid of registration keys / registration counter
- [x] allow users to jump right in without registration by creating a
      auto-generated placeholder account (to be substituted with real info
      if the user converts)

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

# Backlog

Items for a future yet to come.

## Barcode Scanner

~Spend 1 week seeing if we can get a passably good barcode scanner to work.~

This is hard. If you want this feature, let me know! I will probably circle back
to it some day.

## Exercise Tracking

Keep track of calories spent during exercise.

## Consumption Metrics

When a calorie goal is defined, show a progress bar visualization.

## Meal Pacing

Help users understand if they are ahead or behind pace. This should incorporate
the users' plan for the day, for example, planning on eating a large dinner.

## Macro Goals

Allow the input of macro goals (gramsof protein, fat, and carbs).
