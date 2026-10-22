"use strict";

/**
 * Detect constructor *intent*, not merely whether `new fn()` is permitted.
 *
 * semantic (default): ES classes, or ES5-style constructors with additional
 * prototype members. Ordinary function declarations are not constructors
 * solely because their prototype has an inherited `constructor` reference.
 *
 * legacy: preserve the original Nessie behavior for controlled experiments.
 *
 * False negatives are possible: an ES5 constructor with no prototype methods
 * is classified as an ordinary function. The two-valued API metadata cannot
 * express uncertainty; experiments should report this limitation.
 */
function isIntendedConstructor(fn, policy = process.env.NESSIE_CONSTRUCTOR_POLICY || "semantic") {
  if (typeof fn !== "function") return false;

  if (policy === "legacy") {
    try {
      return Boolean(fn.prototype && fn.prototype.constructor);
    } catch (_) {
      return false;
    }
  }
  if (policy !== "semantic") {
    throw new Error(`Unknown NESSIE_CONSTRUCTOR_POLICY: ${policy}`);
  }

  try {
    const source = Function.prototype.toString.call(fn);
    // Handles named/anonymous JS classes, including classes without methods.
    if (/^\s*class(?:\s|\{)/.test(source)) return true;

    const proto = fn.prototype;
    if (!proto || (typeof proto !== "object" && typeof proto !== "function")) {
      return false;
    }
    // Most ES5 constructor functions define methods on their prototype.
    // `constructor` alone does NOT establish constructor intent.
    return Object.getOwnPropertyNames(proto).some((key) => key !== "constructor");
  } catch (_) {
    // Reflection may fail on proxies / host objects. Avoid guessing true.
    return false;
  }
}

module.exports = { isIntendedConstructor };
