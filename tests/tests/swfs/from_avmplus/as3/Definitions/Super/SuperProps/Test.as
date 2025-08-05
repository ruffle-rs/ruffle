/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "8.6.1 Constructor Methods";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Super Property Access";       // Provide ECMA section title or a description
var BUGNUMBER = "";



///////////////////////////////////////////////////////////////
// add your tests here

import SuperPropsPkg.*

import com.adobe.test.Assert;
import com.adobe.test.Utils;
var thisError;
var result;

var spb = new SuperBase();
Assert.expectEq( "sanity check base object 1", "base::staticY", spb.baseProp );
Assert.expectEq( "sanity check base object 2", "base::dynamicX", spb.getBaseVal("x") );
// currently fails - finds undefined dynamic property instead of fixed property:
Assert.expectEq( "sanity check base object 3", "base::staticY", spb.getBaseVal("y") );
spb.setBaseVal("dynamicProp","base::dynamicProp");
Assert.expectEq( "sanity check base object 4", "base::dynamicProp", spb.getBaseVal("dynamicProp") );

var sp = new SuperProps();
Assert.expectEq( "sanity check derived object 1", "base::staticY", sp.inheritedProp );
Assert.expectEq( "sanity check derived object 2", "base::staticY", sp.superPropDot );
// superPropIndex returns X because we cannot test for it returning Y at the moment:
Assert.expectEq( "sanity check derived object 3", "base::dynamicX", sp.superPropIndex );
Assert.expectEq( "sanity check derived object 4", "base::dynamicX", sp.getDerivedVal("x") );
Assert.expectEq( "sanity check derived object 5", "base::dynamicX", sp.getSuperVal("x") );
Assert.expectEq( "sanity check derived object 6", "base::dynamicX", sp.getBaseVal("x") );
// currently fails - finds undefined dynamic property instead of fixed property:
Assert.expectEq( "sanity check derived object 7", "base::staticY", sp.getDerivedVal("y") );
// currently fails - throws exception instead of finding fixed property:
try {
    result = sp.getSuperVal("y");   // super["y"] *should* find base::staticY
} catch (e) {
    result = Utils.referenceError( e.toString() );
} finally {
    Assert.expectEq( "sanity check derived object 8", "base::staticY", result );
}
// currently fails - finds undefined dynamic property instead of fixed property:
Assert.expectEq( "sanity check derived object 9", "base::staticY", sp.getBaseVal("y") );
sp.setDerivedVal("x","derived::x");
Assert.expectEq( "check modified property 1", "base::staticY", sp.inheritedProp );
Assert.expectEq( "check modified property 2", "derived::x", sp.getDerivedVal("x") );
Assert.expectEq( "check base property 1", "base::staticY", sp.superPropDot );
Assert.expectEq( "check base property 2", "derived::x", sp.superPropIndex );
Assert.expectEq( "check base property 3", "derived::x", sp.getSuperVal("x") );
Assert.expectEq( "check base property 4", "derived::x", sp.getBaseVal("x") );

// Test setting super values.  Tests rely on getters functioning properly.
sp.superPropDot = "new dot property value";
Assert.expectEq("Set super value via . property", "new dot property value", sp.superPropDot);

sp.superPropIndex = "new index property value";
Assert.expectEq("Set super value via index property", "new index property value", sp.superPropIndex);

//
////////////////////////////////////////////////////////////////

              // displays results.
