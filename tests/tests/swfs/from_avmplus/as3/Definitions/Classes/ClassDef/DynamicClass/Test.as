/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
import DynamicClassSame.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Access Class Properties & Methods";  // Provide ECMA section title or a description
var BUGNUMBER = "";




var Obj = new DynamicClassAccessor();

// Properties

// Access default property from same package
Assert.expectEq( "Access default property from same package", "1,2,3", Obj.accDefProp().toString() );

// Access internal property from same package
Assert.expectEq( "Access internal property from same package", 100, Obj.accIntProp() );

// Access protected property from same package - NOT LEGAL OUTSIDE DERIVED CLASS
//Assert.expectEq( "Access protected property from same package", -1, Obj.accProtProp() );

// Access public property from same package
Assert.expectEq( "Access public property from same package", 1, Obj.accPubProp() );

// Access private property from same class and package
Assert.expectEq( "Access private property from same class and package", true, Obj.accPrivProp() );

// Access namespace property from same package
Assert.expectEq( "Access namespace property from same package", "nsProp", Obj.accNSProp() );

// Access static public property from same package
Assert.expectEq( "Access public static property from same package", true, Obj.accPubStatProp() );


// Methods


// Access default method from same package
Assert.expectEq( "Access default method from same package", true, Obj.accDefMethod() );

// Access internal method from same package
Assert.expectEq( "Access internal method from same package", 50, Obj.accIntMethod() );

// Access protected method from same package - NOT LEGAL OUTSIDE DERIVED CLASS
//Assert.expectEq( "Access protected method from same package", 1, Obj.accProtMethod() );

// Access public method from same package
Assert.expectEq( "Access public method from same package", true, Obj.accPubMethod() );

// Access private method from same class and package
Assert.expectEq( "Access private method from same class and package", true, Obj.accPrivMethod() );

// Access namespace method from same package
Assert.expectEq( "Access namespace method from same package", "nsMethod", Obj.accNSMethod() );

// Access final public method from same package
Assert.expectEq( "Access final public method from same package", 1, Obj.accPubFinMethod() );

// Access static public method from same package
Assert.expectEq( "Access static public method from same package", 42, Obj.accPubStatMethod() );

// Error cases

// access private property from same package not same class
var thisError = "no error thrown";
try{
    Obj.accPrivPropErr();
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq( "Access private property from same package not same class", "ReferenceError: Error #1065",
                Utils.referenceError( thisError ) );
}

// access private method from same package not same class
thisError = "no error thrown";
try{
    Obj.accPrivMethErr();
} catch (e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq( "Access private method from same package not same class", "ReferenceError: Error #1065",
                Utils.referenceError( thisError ) );
}


              // displays results.
