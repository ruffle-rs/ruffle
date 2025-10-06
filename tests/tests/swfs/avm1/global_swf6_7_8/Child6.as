// Compile with:
//  mtasc -main -header 200:150:30 -version 6 Child6.as -swf child6.swf

class Child6 {
  static function main(current) {
    current.global = _global;
    current.anObject = {};
    current.anArray = [1, 2, 3];
  }
}
