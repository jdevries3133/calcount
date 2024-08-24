// This is teensey bit of JS that we include with ./counter.rs::PrevDayForm.
// It will handle the click event on our three submit buttons;
//
// - breakfast
// - lunch
// - dinner
// - evening
//
// Based on the user's choice, it will take the form field "created_date" and
// turn it into a datetime, putting breakfast at 8:00, lunch at noon, dinner
// at 6pm, and evening at 10pm. Then, the form submit event will propagate,
// and the form will be submitted with the "eaten_at" datetime field filled
// baased on whichever button the user pressed.

const buttons = [
  document.getElementById("breakfast"),
  document.getElementById("lunch"),
  document.getElementById("dinner"),
  document.getElementById("evening"),
];

/**
 * @param {number} hours
 * @returns {number}
 */
function toMillis(hours) {
  return hours * 60 * 1000;
}

/**
 * @param {Date} date
 * @param {number} hours
 * @returns {Date}
 */
function addHours(date, hours) {
  const newDate = new Date(date.getTime());
  newDate.setHours(newDate.getHours() + hours);
  return newDate;
}

/**
 * @param {Event} e
 */
function handler(e) {
  const dateEl = document.getElementById("created_date");
  if (!dateEl.value) {
    // Form submission will be blocked, since this is a required field.
    return;
  }
  const date = new Date(dateEl.value);
  date.setHours(0);
  const datetimes = {
    breakfast: addHours(date, 8),
    lunch: addHours(date, 12),
    dinner: addHours(date, 18),
    evening: addHours(date, 22),
  };
  /**
   * @type {'breakfast' | 'lunch' | 'dinner' | 'evening'}
   */
  let id = e.target.id;
  let datetime = datetimes[id];
  if (!datetime) {
    throw new Error(`could not find datetime for ID ${id}`);
  }
  /**
   * @type {HTMLInputElement}
   */
  const targetEl = document.getElementById("eaten_at");
  targetEl.value = datetime.toISOString();
}

for (const b of buttons) {
  b.addEventListener("click", handler);
}
