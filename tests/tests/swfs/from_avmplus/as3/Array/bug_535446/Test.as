/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


import com.adobe.test.Assert;
import LengthSpoofing.SpoofingArray;
//     var SECTION = "regress";
//     var VERSION = "as3";
//     var TITLE   = "test splice assumes length is correct bug";



//print("concat");
    var foo = new SpoofingArray();
    foo.push(1);
    foo.push(2);
    foo.push(3);
    foo.Spoofing = true;
    Assert.expectEq(
      "test concat spoofing=true",
      "1,2,3,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,4,5,6",
      foo.concat([4,5,6]).toString()
      );
    foo.Spoofing = false;
    Assert.expectEq(
      "test concat spoofing=false",
      "1,2,3,4,5,6",
      foo.concat([4,5,6]).toString()
      );


//print("pop");
    foo = new SpoofingArray();
    foo.push(1);
    foo.push(2);
    foo.push(3);
    foo.Spoofing = true;
    foo.pop();
    Assert.expectEq(
      "test pop spoofing=true",
      100,
      foo.length
      );
    foo.Spoofing = false;
    Assert.expectEq(
      "test pop spoofing=false",
      foo.length,
      2
      );

//print("reverse");
    foo = new SpoofingArray();
    foo.push(1);
    foo.push(2);
    foo.push(3);

    foo.Spoofing = true;
    foo.reverse();
    Assert.expectEq(
      "test reverse spoofing=true",
      100,
      foo.length
      );
    foo.Spoofing = false;
    Assert.expectEq(
      "test reverse spoofing=false",
      3,
      foo.length
      );

//print("shift");
    foo = new SpoofingArray();

    foo.Spoofing = true;
    foo.shift();
    Assert.expectEq(
      "test shift spoofing=true",
      ",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,",
      foo.toString()
      );

    foo = new SpoofingArray();
    foo.push(1);
    foo.push(2);
    foo.push(3);
    foo.Spoofing = false;
    foo.shift();
    Assert.expectEq(
      "test shift spoofing=false",
      "2,3",
      foo.toString()
      );

//  splice: fails
//print("splice");
    foo = new SpoofingArray();

    foo.Spoofing = true;
    foo.splice(90,0,1,2,3,4,5);
    Assert.expectEq(
      "test splice spoofing=true",
      ",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,1,2,3,4,5,,,,,",
      foo.toString()
      );
    foo.Spoofing = false;
    Assert.expectEq(
      "test splice spoofing=false",
      ",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,1,2,3,4,5,,,,,,,,,,",
      foo.toString()
      );
