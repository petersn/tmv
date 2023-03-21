<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.2" name="main_tiles" tilewidth="32" tileheight="32" tilecount="64" columns="8">
 <image source="images/main_tiles.png" width="256" height="256"/>
 <tile id="0" class="nonsolid">
  <properties>
   <property name="name" value="spike"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="4" y="33">
    <polygon points="0,0 12,-26 25,1"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="1" class="marker">
  <properties>
   <property name="name" value="coin"/>
  </properties>
 </tile>
 <tile id="2" class="marker">
  <properties>
   <property name="name" value="rare_coin"/>
  </properties>
 </tile>
 <tile id="3" class="nonsolid">
  <properties>
   <property name="name" value="shooter1"/>
  </properties>
 </tile>
 <tile id="4" class="nonsolid">
  <properties>
   <property name="name" value="save_left"/>
  </properties>
 </tile>
 <tile id="5" class="nonsolid"/>
 <tile id="6" class="nonsolid">
  <properties>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="8" class="marker">
  <properties>
   <property name="name" value="spawn"/>
  </properties>
 </tile>
 <tile id="9" class="nonsolid">
  <properties>
   <property name="name" value="water"/>
  </properties>
 </tile>
 <tile id="10" class="nonsolid">
  <properties>
   <property name="name" value="water"/>
  </properties>
 </tile>
 <tile id="12" class="nonsolid">
  <properties>
   <property name="name" value="platform"/>
  </properties>
 </tile>
 <tile id="13" class="nonsolid">
  <properties>
   <property name="name" value="platform"/>
  </properties>
 </tile>
 <tile id="14" class="nonsolid">
  <properties>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="16" class="marker">
  <properties>
   <property name="name" value="hp_up"/>
  </properties>
 </tile>
 <tile id="17" class="nonsolid">
  <properties>
   <property name="name" value="lava"/>
  </properties>
 </tile>
 <tile id="18" class="nonsolid">
  <properties>
   <property name="name" value="water"/>
  </properties>
 </tile>
 <tile id="19" class="nonsolid"/>
 <tile id="20" class="nonsolid"/>
 <tile id="22" class="nonsolid">
  <properties>
   <property name="count" type="int" value="5"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="23" class="nonsolid">
  <properties>
   <property name="count" type="int" value="5"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="25" class="marker">
  <properties>
   <property name="name" value="stone"/>
  </properties>
 </tile>
 <tile id="26" class="nonsolid">
  <properties>
   <property name="name" value="water"/>
  </properties>
 </tile>
 <tile id="27" class="nonsolid"/>
 <tile id="28" class="nonsolid"/>
 <tile id="30" class="nonsolid">
  <properties>
   <property name="count" type="int" value="10"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="31" class="nonsolid">
  <properties>
   <property name="count" type="int" value="10"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="33" class="marker">
  <properties>
   <property name="name" value="powerup"/>
   <property name="powerup" value="wall_jump"/>
  </properties>
 </tile>
 <tile id="38" class="nonsolid">
  <properties>
   <property name="count" type="int" value="20"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="39" class="nonsolid">
  <properties>
   <property name="count" type="int" value="20"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="41" class="marker">
  <properties>
   <property name="name" value="powerup"/>
   <property name="powerup" value="dash"/>
  </properties>
 </tile>
 <tile id="47">
  <properties>
   <property name="count" type="int" value="10"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
</tileset>
