/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Modify variable after they are created";       // Provide ECMA section title or a description
var BUGNUMBER = "";





import Package1.*;

import com.adobe.test.Assert;
use namespace ns1;

var item1:String = "item1 set at creation time";
var item2 = "item2 set at creation time", item3, item4 = "item4 set at creation time";
var item5:int = 4;

item1 = "item1 modified";
item2 = "item2 modified";
item3 = "item3 modified";
item4 = "item4 modified";
item5 = 3;

Assert.expectEq( "Modify global variable item1", "item1 modified", item1);
Assert.expectEq( "Modify global variable item2", "item2 modified", item2);
Assert.expectEq( "Modify global variable item3", "item3 modified", item3);
Assert.expectEq( "Modify global variable item4", "item4 modified", item4);
Assert.expectEq( "Modify global variable item5", 3, item5);

packageItem1 = "packageItem1 modified";
packageItem2 = "packageItem2 modified";
packageItem3 = "packageItem3 modified";
packageItem4 = "packageItem4 modified";
packageItem5 = 2;

Assert.expectEq( "Modify package variable packageItem1", "packageItem1 modified", packageItem1);
Assert.expectEq( "Modify package variable packageItem2", "packageItem2 modified", packageItem2);
Assert.expectEq( "Modify package variable packageItem3", "packageItem3 modified", packageItem3);
Assert.expectEq( "Modify package variable packageItem4", "packageItem4 modified", packageItem4);
Assert.expectEq( "Modify package variable packageItem5", 2, packageItem5);

var c1 = new Class1();
c1.classItem1 = "Class1 classItem1 modified";
c1.classItem2 = "Class1 classItem2 modified";
c1.classItem3 = "Class1 classItem3 modified";
c1.classItem4 = "Class1 classItem4 modified";
c1.classItem5 = 1;
Class1.classItem6 = "Class1 static classItem6 modified";
c1.classItem7 = "ns1 Class1 classItem7 modified";

Assert.expectEq( "Modify Class1 variable classItem1", "Class1 classItem1 modified", c1.classItem1);
Assert.expectEq( "Modify Class1 variable classItem2", "Class1 classItem2 modified", c1.classItem2);
Assert.expectEq( "Modify Class1 variable classItem3", "Class1 classItem3 modified", c1.classItem3);
Assert.expectEq( "Modify Class1 variable classItem4", "Class1 classItem4 modified", c1.classItem4);
Assert.expectEq( "Modify Class1 variable classItem5", 1, c1.classItem5);
Assert.expectEq( "Modify Class1 variable classItem6", "Class1 static classItem6 modified", Class1.classItem6);
Assert.expectEq( "Modify Class1 variable classItem7", "ns1 Class1 classItem7 modified", c1.classItem7);

var c2 = new Class2();
c2.classItem1 = "Class2 classItem1 modified";
c2.classItem2 = "Class2 classItem2 modified";
c2.classItem3 = "Class2 classItem3 modified";
c2.classItem4 = "Class2 classItem4 modified";
c2.classItem5 = -1;
Class2.classItem6 = "Class2 static classItem6 modified";
c2.classItem7 = "ns1 Class2 classItem7 modified";
Class2.classItem8 = "ns1 Class2 static classItem8 modified";

Assert.expectEq( "Modify Class2 variable classItem1", "Class2 classItem1 modified", c2.classItem1);
Assert.expectEq( "Modify Class2 variable classItem2", "Class2 classItem2 modified", c2.classItem2);
Assert.expectEq( "Modify Class2 variable classItem3", "Class2 classItem3 modified", c2.classItem3);
Assert.expectEq( "Modify Class2 variable classItem4", "Class2 classItem4 modified", c2.classItem4);
Assert.expectEq( "Modify Class2 variable classItem5", -1, c2.classItem5);
Assert.expectEq( "Modify Class2 variable classItem6", "Class2 static classItem6 modified", Class2.classItem6);
Assert.expectEq( "Modify Class2 variable classItem7", "ns1 Class2 classItem7 modified", c2.classItem7);
Assert.expectEq( "Modify Class2 variable classItem8", "ns1 Class2 static classItem8 modified", Class2.classItem8);

              // displays results.
