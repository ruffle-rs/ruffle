/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

// Bug 162570: http://flashqa.macromedia.com/bugapp/detail.asp?ID=162570


//--------------------------------------------------
//--------------------------------------------------

// import same.*;
import test.*;

import com.adobe.test.Assert;

// [NA] This can't compile in mxmlc?
// // same package and class name
// var s:same = new same();
// Assert.expectEq("s is same", true, (s is same));

// static method and parameter with the same name
Assert.expectEq("More.foo(1)", 1, (More.foo(1), More.a));

// instance method and parameter with the same name
var m:More = new More();
Assert.expectEq("m.bar(true)", true, (m.bar(true), m.b));

// instance class and method with the same name
var bar:More = new More();
Assert.expectEq("bar.bar(true)", true, (bar.bar(true), bar.b));

// instance class and property with the same name
var b:More = new More();
Assert.expectEq("b.bar(true)", true, (b.bar(true), b.b));

// dynamic method and parameter with the same name
dynamic class C {}
var c:C = new C();
c.a = false;
c.b = function (b:Boolean):void { c.a = b; }
Assert.expectEq("c.b(true)", true, (c.b(true), c.a));

//--------------------------------------------------
