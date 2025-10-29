
() @namespace(ui) =>

  local (self) @AutoRender @StatedComponent({
    items = Vec()
  }) @UIOverride('prepare', function(self)
    async(function()
      sleep(1)
      self.items:set(Vec({ "a", "b", "c", "d" }))
    end)
  end) @Component AppRoot =>
    return Frame {
      children = {
        Spinner {
          inactive = self.items:has_items(),
        },
        VList {
          inactive = self.items:is_empty(),
          items = self.items,
          render = function(e)
            return Label {
              text = e
            }
          end
        }
      }
    }
  end

end