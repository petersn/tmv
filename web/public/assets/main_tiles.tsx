<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.9" tiledversion="1.9.2" name="main_tiles" tilewidth="32" tileheight="32" tilecount="128" columns="8">
 <image source="images/main_tiles.png" width="256" height="512"/>
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
 <tile id="21" class="marker">
  <properties>
   <property name="name" value="moving_platform"/>
  </properties>
 </tile>
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
 <tile id="29" class="marker">
  <properties>
   <property name="name" value="vanish_block"/>
  </properties>
 </tile>
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
 <tile id="34" class="marker">
  <properties>
   <property name="name" value="thwump"/>
  </properties>
 </tile>
 <tile id="35" class="marker">
  <properties>
   <property name="name" value="turn_laser"/>
  </properties>
 </tile>
 <tile id="36" class="nonsolid">
  <properties>
   <property name="name" value="shooter2"/>
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
 <tile id="43" class="nonsolid"/>
 <tile id="44" class="nonsolid"/>
 <tile id="46" class="nonsolid">
  <properties>
   <property name="count" type="int" value="15"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="47" class="nonsolid">
  <properties>
   <property name="count" type="int" value="15"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="49" class="marker">
  <properties>
   <property name="name" value="powerup"/>
   <property name="powerup" value="water"/>
  </properties>
 </tile>
 <tile id="50" class="marker">
  <properties>
   <property name="name" value="powerup"/>
   <property name="powerup" value="small"/>
  </properties>
 </tile>
 <tile id="51" class="nonsolid"/>
 <tile id="52" class="nonsolid"/>
 <tile id="54" class="nonsolid">
  <properties>
   <property name="count" type="int" value="30"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="55" class="nonsolid">
  <properties>
   <property name="count" type="int" value="30"/>
   <property name="name" value="coin_wall"/>
  </properties>
 </tile>
 <tile id="57" class="marker">
  <properties>
   <property name="name" value="powerup"/>
   <property name="powerup" value="lava"/>
  </properties>
 </tile>
 <tile id="58" class="marker">
  <properties>
   <property name="name" value="powerup"/>
   <property name="powerup" value="double_jump"/>
  </properties>
 </tile>
 <tile id="64" class="nonsolid">
  <properties>
   <property name="name" value="beehive"/>
  </properties>
 </tile>
 <tile id="65" class="nonsolid"/>
 <tile id="67" class="nonsolid"/>
 <tile id="68" class="nonsolid"/>
 <tile id="72" class="nonsolid"/>
 <tile id="73" class="nonsolid"/>
 <tile id="75" class="nonsolid"/>
 <tile id="76" class="nonsolid"/>
</tileset>
