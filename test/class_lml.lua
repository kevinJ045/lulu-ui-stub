
local function Item(props)
  return lml! {
    <label text={props.text} />
  }
end

local class! @Component @StatedComponent({
  loading = true,
  fetched_text = "",
  objects = Vec({"a", "b", "c"}),
  selected = "volvo"
}) @ComponentValues({
  selectables = {volvo = "Volvo", marcedes = "Marcedes", ferrari = "Ferrari"}
}) @AutoRender Body:Widget, {
  prepare(){
    async(function()
      sleep(1)
      self.loading:set(false)
    end)
  }
  rebuild(){
    -- override rebuild
    -- disable re-render on variable change
  }
  build(){
    return lml! {
      <style>
        <frame inactive={self.loading:inverse()} style={{
          width = "fill",
          height = "70%",
        }}>
          <align layout="center_both">
            <spinner />
          </align>
        </frame>
        <vbox inactive={self.loading}>
          <heading text="The Heading Text" />
          <link text={self.selected:inside(self.selectables)} />
          <image inactive={self.selected:is_not("volvo")} src="/home/makano/Pictures/coffeescript_waifu2x_art_noise3_scale.png" fit_to={{100, 100}} />
          <combobox selected={self.selected} values={self.selectables} />
          <each items={self.objects:get()} render={function(item)
            return <Item text={item} />
          end} />
        </vbox>
      </style>
    }
  }
}
