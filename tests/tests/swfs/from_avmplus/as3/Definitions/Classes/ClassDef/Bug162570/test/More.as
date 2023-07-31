/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package test {
public class More {
  function More() {}

  public static var a:int = 0;
  public static function foo(foo:int):void { More.a = foo; }

  public var b:Boolean = false;
  public function bar(bar:Boolean):void { b = bar; }
}
}