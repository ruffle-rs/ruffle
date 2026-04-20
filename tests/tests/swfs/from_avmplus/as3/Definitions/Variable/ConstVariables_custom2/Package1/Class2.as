/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package Package1
{
    public class Class2
    {
        public const classItem1;
        public const classItem2, classItem3, classItem4;
        public const classItem5:int = 6;
        public static const classItem6 = init();
        ns1 const classItem7:String;
        ns1 static const classItem8:String = init2();

        public function Class2()
        {
            classItem1 = "const Class2 classItem1 set in constructor";
            classItem2 = "const Class2 classItem2 set in constructor";
            classItem4 = "const Class2 classItem4 set in constructor";
            classItem5 = 7;
            //ns1::classItem7 = "ns1 const Class2 classItem7 set in constructor";
        }

        public static function init()
        {
            return "static const Class2 classItem6 set in function";
        }

        public static function init2()
        {
            return "ns1 static const Class2 classItem8 set in function";
        }
    }
}
