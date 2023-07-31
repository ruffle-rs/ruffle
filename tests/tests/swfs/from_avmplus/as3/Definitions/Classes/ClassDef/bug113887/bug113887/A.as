/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// bug 113887: static initialization using the class itself
package bug113887 {

class A {
  static var obj: A = new A();
  var init: String = "no";

  function A() {
    init = "yes";
  }

  function wasInitialized(): String {
    return init;
  }
}
}