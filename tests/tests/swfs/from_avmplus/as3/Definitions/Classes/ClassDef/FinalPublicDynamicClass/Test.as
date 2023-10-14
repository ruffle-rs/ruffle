/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import FinalPublicDynamicClassPackage.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Access Class Properties & Methods";  // Provide ECMA section title or a description
var BUGNUMBER = "";




var arr = new Array(1,2,3);
var arr2 = new Array(3,2,1);
var Obj = new FinalPublicDynamicClass();
var d = new Date(0);
var d2 = new Date(1);
var f = new Function();
var str = "Test";
var ob = new Object();

// ********************************************
// Access Default method
// ********************************************
Assert.expectEq( "*** Access default method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setArray(arr), Obj.getArray()", arr, Obj.testGetSetArray(arr) );

// ********************************************
// Access Default virtual method
// ********************************************
Assert.expectEq( "*** Access default virtual method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setVirtArray(arr), Obj.getVirtArray()", arr2, Obj.testGetSetVirtualArray(arr2) );

// ********************************************
// Access Default Static method
// ********************************************
Assert.expectEq( "*** Access static method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setStatFunction(f), Obj.getStatFunction()", f, Obj.testGetSetStatFunction(f) );

// ********************************************
// Access Default Final method
// ********************************************
Assert.expectEq( "*** Access final method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setFinNumber(10), Obj.getFinNumber()", 10, Obj.testGetSetFinNumber(10) );

// ********************************************
// Access Internal method
// ********************************************
Assert.expectEq( "*** Access internal method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setInternalArray(arr), Obj.getInternalArray()", arr, Obj.testGetSetInternalArray(arr) );

// ********************************************
// Access Internal virtual method
// ********************************************
Assert.expectEq( "*** Access internal virtual method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setInternalVirtualArray(arr), Obj.getInternalVirtualArray()", arr2, Obj.testGetSetInternalVirtualArray(arr2) );


// ********************************************
// Access Internal Static method
// ********************************************
Assert.expectEq( "*** Access internal static method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setInternalStatFunction(f), Obj.getInternalStatFunction()", f, Obj.testGetSetInternalStatFunction(f) );


// ********************************************
// Access Internal Final method
// ********************************************
Assert.expectEq( "*** Access internal final method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setInternalFinNumber(10), Obj.getInternalFinNumber()", 10, Obj.testGetSetInternalFinNumber(10) );

// ********************************************
// Access Private method
// ********************************************
Assert.expectEq( "*** Access private method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPrivDate(date), Obj.getPrivDate()", d.getFullYear(), Obj.testGetSetPrivDate(d).getFullYear() );

// ********************************************
// Access Private virtual method
// ********************************************
Assert.expectEq( "*** Access private virtual method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPrivVirtualDate(date), Obj.getPrivVirtualDate()", d2.getFullYear(), Obj.testGetSetPrivVirtualDate(d2).getFullYear() );

// ********************************************
// Access Private Static method
// ********************************************
Assert.expectEq( "*** Access private static method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPrivStatString(s), Obj.getPrivStatString", str, Obj.testGetSetPrivStatString(str) );

// ********************************************
// Access Private Final method
// ********************************************
Assert.expectEq( "*** Access private final method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPrivFinalString(s), Obj.getPrivFinalString", str, Obj.testGetSetPrivFinalString(str) );

// ********************************************
// Access Public method
// ********************************************
Assert.expectEq( "*** Access public method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPubBoolean(b), Obj.getPubBoolean()", true, (Obj.setPubBoolean(true), Obj.getPubBoolean()) );

// ********************************************
// Access Public virtual method
// ********************************************
Assert.expectEq( "*** Access public virtual method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPubVirtualBoolean(b), Obj.getPubVirtualBoolean()", false, (Obj.setPubBoolean(false), Obj.getPubBoolean()) );

// ********************************************
// Access Public Static method
// ********************************************
Assert.expectEq( "*** Access public static method of a class ***", 1, 1 );
Assert.expectEq( "FinalPublicDynamicClass.setPubStatObject(ob), FinalPublicDynamicClass.getPubStatObject()", ob, (FinalPublicDynamicClass.setPubStatObject(ob), FinalPublicDynamicClass.getPubStatObject()) );

// ********************************************
// Access Public Final method
// ********************************************
Assert.expectEq( "*** Access public final method of a class ***", 1, 1 );
Assert.expectEq( "Obj.setPubFinArray(arr), Obj.getPubFinArray()", arr, (Obj.setPubFinArray(arr), Obj.getPubFinArray()) );





// ********************************************
// Access Public property
// ********************************************
Assert.expectEq( "*** Access public property of a class ***", 1, 1 );
Assert.expectEq( "Obj.pubBoolean = true, Obj.pubBoolean", true, (Obj.pubBoolean = true, Obj.pubBoolean) );

// ********************************************
// Access Public Static property
// ********************************************
Assert.expectEq( "*** Access public satic property of a class ***", 1, 1 );
Assert.expectEq( "FinalPublicDynamicClass.pubStatObject = ob, FinalPublicDynamicClass.pubStatObject", ob, (FinalPublicDynamicClass.pubStatObject = ob, FinalPublicDynamicClass.pubStatObject) );

// ********************************************
// Access Public Final property
// ********************************************
Assert.expectEq( "*** Access public final property of a class ***", 1, 1 );
Assert.expectEq( "Obj.pubFinArray = arr, Obj.pubFinArray", arr, (Obj.pubFinArray = arr, Obj.pubFinArray) );

// ********************************************
// Access Public Final Static property
// ********************************************
Assert.expectEq( "*** Access public final static property of a class ***", 1, 1 );
Assert.expectEq( "FinalPublicDynamicClass.pubFinalStaticNumber = 10, FinalPublicDynamicClass.pubFinalStaticNumber", 10, (FinalPublicDynamicClass.pubFinalStaticNumber = 10, FinalPublicDynamicClass.pubFinalStaticNumber) );



              // displays results.
