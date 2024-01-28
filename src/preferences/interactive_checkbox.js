/**
 * Disable the min and max daily limit fields related to calorie balancing
 * when calorie balancing is disabled. Also, show some helper text in this
 * case.
 */

const checkbox = () =>
  document.querySelector('[name="calorie_balancing_enabled"]');
const minField = () =>
  document.querySelector('[name="calorie_balancing_min_calories"]');
const maxField = () =>
  document.querySelector('[name="calorie_balancing_max_calories"]');

/**
 * @param {HTMLInputElement} el
 */
function disable(el) {
  el.disabled = true;
  el.readOnly = true;
  el.classList.add("!bg-gray-400");
  const help = document.createElement("p");
  help.innerText = "Field is disabled because calorie balancing is disabled.";
  help.classList.add(
    "text-yellow-700",
    "italic",
    "text-xs",
    "dark:text-yellow-200",
  );
  help.id = "help-text";
  el.after(help);
}

/**
 * @param {HTMLInputElement} el
 */
function enable(el) {
  el.disabled = false;
  el.readOnly = false;
  el.classList.remove("!bg-gray-400");
  el.parentNode.querySelector("#help-text")?.remove();
}

checkbox().addEventListener("click", (e) => {
  if (e.target.checked) {
    enable(minField());
    enable(maxField());
  } else {
    disable(minField());
    disable(maxField());
  }
});

// Also, disable the fields on load if calorie balancing is not enabled.
if (!checkbox().checked) {
  disable(minField());
  disable(maxField());
}
