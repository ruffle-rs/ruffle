/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// bug 113887: static initialization using the class itself
package bug113887 {

class B {
  static function createFoo(): B {
    return new B();
  }

  static var foo: B = B.createFoo();
  var init: String = "no";

  function B() {
    init = "yes";
  }

  function wasInitialized(): String {
    return init;
  }
}
}