/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "protected variables";       // Provide ECMA section title or a description
var BUGNUMBER = "";



import Package1.*;

import com.adobe.test.Assert;
var c1 = new Class1();
c1.setClassItem1("Modified Class1 classItem1");
Assert.expectEq( "Class1 protected variable classItem1", "Modified Class1 classItem1", c1.getClassItem1());
Assert.expectEq( "Class1 protected const variable classItem2", "Class1 protected const classItem2 set at creation time", c1.getClassItem2());

Class1.setClassItem3("Modified Class1 classItem3");

Assert.expectEq( "Class1 protected static variable classItem3", "Modified Class1 classItem3", Class1.getClassItem3());
Assert.expectEq( "Class1 protected static const variable classItem4", "Class protected static const classItem4 set at creation time", Class1.getClassItem4());

var c2 = new Class2();

Assert.expectEq( "Class2 protected variable classItem1", "Class1 protected var classItem1 set at creation time", c2.getInheritedClassItem1());
Assert.expectEq( "Class2 protected const variable classItem2", "Class1 protected const classItem2 set at creation time", c2.getInheritedClassItem2());
Assert.expectEq( "Class2 protected static variable classItem3", "Modified Class1 classItem3", c2.getInheritedClassItem3());
Assert.expectEq( "Class2 protected static const variable classItem4", "Class protected static const classItem4 set at creation time", c2.getInheritedClassItem4());

              // displays results.
