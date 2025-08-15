/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;


var myXML:XML = new XML('<Test><KeyFrame name="&kColor_TextEditTextFieldOutlineNormalEnd;" t="0.0" v="&kTextEditInnerShadow_DarkV;" /></Test>');
var expected:String = '<Test><KeyFrame name="&amp;kColor_TextEditTextFieldOutlineNormalEnd;" t="0.0" v="&amp;kTextEditInnerShadow_DarkV;"/></Test>';


var pp:Boolean = XML.prettyPrinting;

XML.prettyPrinting = false;

Assert.expectEq('Bug 535882 -  XMLParser incorrectly converts attribute values containing entities', expected, myXML.toXMLString());

XML.prettyPrinting = pp;  // restore prettyPrinting setting

