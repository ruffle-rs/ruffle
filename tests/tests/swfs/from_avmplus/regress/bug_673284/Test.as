/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Regression Tests";
// var VERSION = "AS3";
// var TITLE   = "Bug 673284";
// var bug = "673284";



var subject:String="AAA";
for (var i = 0; i < 10; i ++) {
    subject.match(/(?i)a(?-i)(.?)|c/g);
    subject.match(/(?i)a(?-i).?|c/g);
    subject.match(/(?i)a(?-i)(.??)|c/g);
    subject.match(/(?i)a(?-i).??|c/g);
    subject.match(/(?i)a(?-i)(\p{Z}[^\p{C}\p{Z}]+)*|c/g);
    subject.match(/(?i)a(?-i)[^\p{C}\p{Z}]+|c/g);

    subject.match(/(?i)a(?-i)[\pL](abc)(?1)|c/g); // overwrite #1

    subject.match(/(?i)a(?-i)[\pL]=(abc))*X|c/g);
    subject.match(/(?i)a(?-i)(?:\p{Lu}|\x20)+|c/g);
    subject.match(/(?i)a(?-i)[\p{Lu}\x20]+|c/g);
    subject.match(/(?i)a(?-i)(a.b(?s)c.d|x.y)|c/g);
    subject.match(/(?i)a(?-i)\d?|c/g);
    subject.match(/(?i)a(?-i)[abcd]*|c/g);
    subject.match(/(?i)a(?-i)[abcd]+|c/g);
    subject.match(/(?i)a(?-i)[abcd]?|c/g);
    subject.match(/(?i)a(?-i)[abcd]{2,3}|c/g);

    subject.match(/(?i)a(?-i)(abc)*|c/g); // overwrite #2

    subject.match(/(?i)a(?-i)(abc)+|c/g);
    subject.match(/(?i)a(?-i)(abc)?|c/g);        // overwrites
    subject.match(/(?i)a(?-i)(a*\w|ab)|c/g);
    subject.match(/(?i)a(?-i)(?<=abc|xy)|c/g);
    subject.match(/(?i)a(?-i)(a*|xyz)|c/g);
    subject.match(/(?i)a(?-i)(ab*(cd|ef))+|c/g); // overwrites
    subject.match(/(?i)a(?-i)(a+|b+|c+)*|c/g);
    subject.match(/(?i)a(?-i)(?<=a|bbbb)|c/g);
    subject.match(/(?i)a(?-i)[^\r\n]{6,}|c/g);
    subject.match(/(?i)a(?-i)[^a]{6,}|c/g);
    subject.match(/(?i)a(?-i)^\pN{2,3}|c/g);
    subject.match(/(?i)a(?-i)bac/g);
    subject.match(/(?i)a(?-i)b\tc/g);
    subject.match(/(?i)a(?-i)ba*c/g);
    subject.match(/(?i)a(?-i)bc?c/g);
    subject.match(/(?i)a(?-i)bz+c/g);
    subject.match(/(?i)a(?-i)br{3}c/g);
    subject.match(/(?i)a(?-i)bb{2,}c/g);
    subject.match(/(?i)a(?-i)by{4,5}c/g);
    subject.match(/(?i)a(?-i)b^c/g);
    subject.match(/(?i)a(?-i)b(abc){1,2}c/g);
    subject.match(/(?i)a(?-i)b(b+?|a){1,2}?c/g);
    subject.match(/(?i)a(?-i)bb+?c/g);
    subject.match(/(?i)a(?-i)b(b+|a){1,2}c/g);
    subject.match(/(?i)a(?-i)b(b+|a){1,2}?c/g);
    subject.match(/(?i)a(?-i)b(b*|ba){1,2}?c/g);
    subject.match(/(?i)a(?-i)b(ba|b*){1,2}?c/g);
    subject.match(/(?i)a(?-i)b\c/g);
    /* bug http://watsonexp.corp.adobe.com/#bug=3345099
    subject.match(/(?i)a(?-i)b[c/g);
    */
    subject.match(/(?i)a(?-i)bc{c/g);
    subject.match(/(?i)a(?-i)b[ab\]cde]c/g);
    subject.match(/(?i)a(?-i)b[]cde]c/g);
    subject.match(/(?i)a(?-i)b[0-9]+c/g);
    subject.match(/(?i)a(?-i)b.*c/g);

}

Assert.expectEq("Completed", true, true);



