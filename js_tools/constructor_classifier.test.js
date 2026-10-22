"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const { isIntendedConstructor } = require("./constructor_classifier");

function readJson(filePath, callback) {
  if (callback) callback(null, filePath);
}
function writeJson(filePath, data, callback) {
  if (callback) callback(null, data);
}

class EmptyClass {}
class OrdinaryClass { method() { return 1; } }
function LegacyWidget() { this.count = 0; }
LegacyWidget.prototype.inc = function () { this.count++; };
function OldStyleNoMethods() { this.count = 0; }

test("ordinary exported API functions are not misclassified", () => {
  assert.equal(isIntendedConstructor(readJson), false);
  assert.equal(isIntendedConstructor(writeJson), false);
});

test("ES classes are identified even when no methods are declared", () => {
  assert.equal(isIntendedConstructor(EmptyClass), true);
  assert.equal(isIntendedConstructor(OrdinaryClass), true);
});

test("ES5 prototype-method constructors remain supported", () => {
  assert.equal(isIntendedConstructor(LegacyWidget), true);
});

test("arrow functions and non-functions remain non-constructors", () => {
  assert.equal(isIntendedConstructor(() => {}), false);
  assert.equal(isIntendedConstructor(null), false);
});

test("conservative policy documents ES5 constructors without methods", () => {
  assert.equal(isIntendedConstructor(OldStyleNoMethods), false);
});

test("legacy policy reproduces the prior prototype-based flag", () => {
  assert.equal(isIntendedConstructor(readJson, "legacy"), true);
  assert.equal(isIntendedConstructor(OrdinaryClass, "legacy"), true);
});

test("unknown policy is rejected rather than silently changing experiments", () => {
  assert.throws(() => isIntendedConstructor(readJson, "unknown"), /Unknown NESSIE_CONSTRUCTOR_POLICY/);
});
