/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {
  public static const vSelfConst:Vector.<Test> = new Vector.<Test>();
  public static var vSelfVar:Vector.<Test> = new Vector.<Test>();
  public static var vSelfLiteral:Vector.<Test> = new Vector.<Test>([undefined]);
  public static var vInitStatic:Vector.<Test>;

  {
    vInitStatic = new Vector.<Test>();
  }

  public function Test()
  {
    vSelfConst.push(this);
    vSelfVar.push(this);
    vInitStatic.push(this);
    vSelfLiteral[0] = this;
  }
}
}

import com.adobe.test.Assert;
// var SECTION = "";
// var VERSION = "ECMA_1";
// var TITLE   = "staticInitializer";


var t:Test = new Test();

Assert.expectEq("const vSelfConst:Vector.<Test>", "[object Test]", Test.vSelfConst[0].toString() );
Assert.expectEq("var vSelfVar:Vector.<Test>", "[object Test]", Test.vSelfVar[0].toString() );
Assert.expectEq("var vSelfLiteral:Vector.<Test>", "[object Test]", Test.vSelfLiteral[0].toString() );
Assert.expectEq("var vInitStatic:Vector.<Test>", "[object Test]", Test.vInitStatic[0].toString() );
