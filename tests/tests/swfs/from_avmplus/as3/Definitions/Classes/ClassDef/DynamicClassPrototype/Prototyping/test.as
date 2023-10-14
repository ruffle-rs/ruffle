/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package Prototyping {
import DynamicClassPrototype.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

public function test() {

// var SECTION = "Class Prototype";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing prototype in dynamic classes";  // Provide ECMA section title or a description
//var BUGNUMBER = "";



  var dynamicObj = new DynamicClass();



  DynamicClass.prototype.array = new Array('a', 'b', 'c');
  Assert.expectEq( "Try overriding default property through a dynamic class' prototype object", "1,2,3", dynamicObj.accessDefaultProperty().toString() );

  DynamicClass.prototype.intNumber = 500;
  Assert.expectEq( "Try overriding internal property through a dynamic class' prototype object", "100", dynamicObj.intNumber.toString() );

  DynamicClass.prototype.protInt = 0; // Note: this works because the protected property is not visible!
  Assert.expectEq( "Try overriding protected property through a dynamic class' prototype object", "0", dynamicObj.protInt.toString() );

  DynamicClass.prototype.pubUint = 0;
  Assert.expectEq( "Try overriding public property through a dynamic class' prototype object", "1", dynamicObj.pubUint.toString() );

  DynamicClass.prototype.privVar = false;
  Assert.expectEq( "Try overriding private property through a dynamic class' prototype object", "true", dynamicObj.accPrivProp().toString() );

  DynamicClass.prototype.pubStat = 200;
  Assert.expectEq( "Try overriding public static property through a dynamic class' prototype object", "100", DynamicClass.pubStat.toString() );

  DynamicClass.prototype.nsProp = "fakeNS";
  Assert.expectEq( "Try overriding namespace property through a dynamic class' prototype object", "nsProp", dynamicObj.accNS().toString() );

  DynamicClass.prototype.DynamicMethod = false;
  Assert.expectEq( "Try overriding Dynamic method through a dynamic class' prototype object", true, dynamicObj.defaultMethod() );

  DynamicClass.prototype.internalMethod = -1;
  Assert.expectEq( "Try overriding internal method through a dynamic class' prototype object", 1, dynamicObj.internalMethod() );

//DynamicClass.prototype.protectedMethod = -1;
//Assert.expectEq( "Try overriding protected method through a dynamic class' prototype object", 1, dynamicObj.protectedMethod() );

  DynamicClass.prototype.publicMethod = false;
  Assert.expectEq( "Try overriding public method through a dynamic class' prototype object", true, dynamicObj.publicMethod() );

  DynamicClass.prototype.privateMethod = false;
  Assert.expectEq( "Try overriding private method through a dynamic class' prototype object", true, dynamicObj.accPrivMethod() );

  DynamicClass.prototype.nsMethod = -1;
  Assert.expectEq( "Try overriding namespace method through a dynamic class' prototype object", 1, dynamicObj.accNSMethod() );

  DynamicClass.prototype.publicFinalMethod = -1;
  Assert.expectEq( "Try overriding public final method through a dynamic class' prototype object", 1, dynamicObj.publicFinalMethod() );

  DynamicClass.prototype.publicStaticMethod = -1;
  Assert.expectEq( "Try overriding public static method through a dynamic class' prototype object", 42, DynamicClass.publicStaticMethod() );


  DynamicClass.prototype.newArray = new Array('a', 'b', 'c');
  Assert.expectEq( "Try adding new property through a dynamic class' prototype object", "a,b,c", dynamicObj.newArray.toString() );

  DynamicClass.prototype.testFunction = function () {return true};
  Assert.expectEq("Try adding new method through a dynamic class' prototype object", true, dynamicObj.testFunction());

  var equivalent:Boolean = (DynamicClass.prototype.constructor == DynamicClass);
  Assert.expectEq("Verify prototype constructor is equivalent to class object", true, equivalent);


  var thisError10 = "no error thrown";
  var temp:Object = new Object();
  try{
    DynamicClass.prototype = temp;
  } catch (e) {
    thisError10 = e.toString();
  } finally {
    Assert.expectEq( "Try to write to FinalClass' prototype object", "ReferenceError: Error #1074",
      Utils.referenceError( thisError10 ) );
  }

  // displays results.

  }

}
