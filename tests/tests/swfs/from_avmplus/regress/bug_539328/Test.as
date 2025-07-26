/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}



    interface IBar
    {
        function i0():*;
    }

import avmplus.*;
import com.adobe.test.Assert;


const x = 20/2;

var expected:XML = <type name="IBar" base="Class" isDynamic="true" isFinal="true" isStatic="true">
  <extendsClass type="Class"/>
  <extendsClass type="Object"/>
  <accessor name="prototype" access="readonly" type="*" declaredBy="Class"/>
  <factory type="IBar">
    <method name="i0" declaredBy="IBar" returnType="*" uri="IBar"/>
  </factory>
</type>



Assert.expectEq("Bug 539328: describeType() omits interface methods", expected.toString(), describeType(IBar, FLASH10_FLAGS).toString());

