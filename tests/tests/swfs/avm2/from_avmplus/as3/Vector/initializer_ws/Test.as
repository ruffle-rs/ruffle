/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/**
 File Name:    initializerWs.as
 Description:
 Test whitespace behavior of Vector initializers.
 */
// var SECTION="";
// var VERSION = "ECMA_1";



var x:Vector.<int> =


  new
  <
    int
    >
  [
    1
    ,
    2
    ,
    3
  ];

Assert.expectEq("spraddled initializer",
  "1,2,3",
  x.toString());

var v:Vector.<XML> = new <  XML
    >
  [
    <myXml>
      <test></test>
    </myXml>,
    <body><item1>
    </item1><item2 title="title">
    </item2></body>
  ]
;

Assert.expectEq("spraddled initializer using xml",
  '<myXml>  <test/></myXml>,<body>  <item1/>  <item2 title="title"/></body>',
  v.toString().split('\n').join(''));
