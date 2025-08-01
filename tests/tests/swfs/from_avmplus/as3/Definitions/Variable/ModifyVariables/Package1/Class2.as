/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package Package1
{
    public class Class2
    {
        public var classItem1;
        public var classItem2, classItem3, classItem4;
        public var classItem5:int = 6;
        public static var classItem6 = init();;
        ns1 var classItem7;
        ns1 static var classItem8 = init2();

        public function Class2()
        {
            classItem1 = "Class2 classItem1 set in constructor";
            classItem2 = "Class2 classItem2 set in constructor";
            classItem4 = "Class2 classItem4 set in constructor";
            classItem5 = 7;
            classItem7 = "ns1 Class2 classItem7 set in constructor";
        }

        public static function init()
        {
            return "static Class2 classItem6 set in static function";
        }

        public static function init2()
        {
            return "ns1 static Class2 classItem8 set in static function";
        }
    }
}
