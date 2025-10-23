
local id = 0
local elements = Vec({})

class! @into_collectible("collect") State(@default_to("") value), {
  format(str){
    return self:map(function(e)
      return str:gsub("{}", e):gsub("{{}}", "{}")
    end)
  }

  inverse(){
    return self:map(function(e)
      return not e
    end)
  }
  
  map(formatter){
    local s = State(formatter(self.value))
    self.on_set = function(val)
      s.value = formatter(val)
    end
    return s
  }

  get(){
    return self.value
  }

  set(val){
    self.value = val
    if self.on_set then
      self.on_set(val)
    end
    return self
  }

  add(val){
    if type(self.value) == "string" then
      self:set(self.value .. val)
    elseif type(self.value) == "number" then
      self:set(self.value + val)
    elseif instanceof(self.value, Vec) then
      self.value.push(val)
      self:set(self.value)
    elseif type(self.value) == "table" then
      table.insert(self.value, val)
      self:set(self.value)
    end
    return self
  }
  
  sub(val){
    if type(self.value) == "number" then
      self.value = self.value - val
    end
  }
}

function remove_node_from(children, node)
  children:remove(function(child)
    if type(node) == "number" then
      return child.id == node
    else
      return child.id == node.id
    end
  end)
end

class! @into_collectible("collect") Node(@default_to("") #name, @default_to(Vec()) #children, @default_to({}) #props), {
  init(){
    if not instanceof(self.children, Vec) then
      self.children = Vec(self.children)
    end
    self.id = id
    id = id + 1

    local raw_props = self.props
    local proxy = {}

    setmetatable(proxy, {
      __index = function(_, key)
        local val = raw_props[key]
        if val ~= nil and type(val) == "table" and instanceof(val, State) then
          return val:get()
        end
        if key == "__real" then
          return raw_props
        end
        return val
      end,

      __newindex = function(_, key, val)
        if raw_props[key] ~= nil and type(raw_props[key]) == "table" and instanceof(raw_props[key], State) then
          raw_props[key]:set(val)
        else
          raw_props[key] = val
        end
      end,

      __pairs = function(t)
        return function(_, k)
          local next_key, next_val = next(t, k)
          if next_val ~= nil and instanceof(next_val, State) then
            return next_key, next_val:get()
          end
          return next_key, next_val
        end, t, nil
      end,

      __len = function()
        return #raw_props
      end,
    })

    self.props = proxy
  }

  add(...){
    for_each! {_,v}, {{...}}, {
      if instanceof(v, Vec) then
        v.items:for_each(function(v)
          v.parent = self
        end)
        self.children:extend(v.items)
      elseif instanceof(v, Node) then
        self.children:push(v)
        v.parent = self
      end
    }
    return self
  }

  remove_node(node){
    remove_node_from(self.children, node)
    return self
  }

  remove(){
    if self.parent == "root" then
      remove_node_from(elements, node)
    else
      self.parent.remove_node(self)
    end
  }

  find(id, index){
    if not index then index = 0 end
    local _, value =  self.children:find(function(item, i)
      if i < index then return false end
      if type(id) == "string" then
        return item.name == id
      else
        return item.id == id
      end
    end)
    return value
  }
  
  findAll(name, index){
    if not index then index = 0 end
    return self.children:filter(function(item, i)
      if i < index then return false end
      return item.name == name
    end)
  }

  into_root(){
    self.parent = "root"
    elements:push(self)
    return self
  }
}

class! Widget:Node, {
  init(){
    self._event_handlers = Vec()
    for k, v in pairs(self.props.__real) do
      if k:sub(1, 3) == "on_" then
        local event = k:sub(4, #k)
        self:on(event, v)
        self.props[k] = nil
      end
    end
  }
  on(event, fn){
    self._event_handlers:push(collect! { event, fn })
    return self
  }
  off(event){
    self._event_handlers:remove(function(v)
      if type(event) == "function" then
        return v.fn == event
      else
        return v.event == event
      end
    end)
    return self
  }
  emit(event, data){
    self._event_handlers:for_each(function(v)
      if v.event == event then
        v.fn(self, data)
      end
    end)
  }
  set(prop, val){
    self.props[prop] = val
    return self
  }
  get(prop){
    return self.props[prop]
  }
  _render(){}
}

local default_elements = {}

function widget_defaults(defaults)
  return function(_class)

    function _class:init(o)
      for k, v in pairs(defaults) do
        if self.props[k] == nil then
          self.props[k] = v
        end
      end
    end
    
    return _class
  end
end


local function handle_style(self, ui)
  if self.props.style then
    ui:set_style(self.props.style, elements:len() < 2)
  end
end

local function register_element(name, options_default, render)
  local class! @widget_defaults(options_default) _c:Widget, {
    _render(ui){
      render(self, ui)
    }
  }

  default_elements[name] = function(props, children)
    return _c collect! {
      name,
      props,
      children
    }
  end

  return default_elements[name]
end


ui = {}

local function handle_events(self, event, response)
  if response[event] then
    self:emit(event, response.value)
  end
end

local function handle_change(self, name, response)
  if response.changed then
    self.props[name] = response.value
  end

  return response
end

local function handle_reponse(self, response)
  self.is_pointer_button_down_on = response.is_pointer_button_down_on
  self.drag_delta = response.drag_delta
  self.contains_pointer = response.contains_pointer
  self.pointer_pos = function()
    return response.interact_pointer_pos
  end

  handle_events(self, "clicked", response)
  handle_events(self, "middle_clicked", response)
  handle_events(self, "double_clicked", response)
  handle_events(self, "triple_clicked", response)
  handle_events(self, "clicked_elsewhere", response)
  handle_events(self, "lost_focus", response)
  handle_events(self, "gained_focus", response)
  handle_events(self, "has_focus", response)
  handle_events(self, "hovered", response)
  handle_events(self, "changed", response)
  handle_events(self, "highlighted", response)
  handle_events(self, "long_touched", response)
  handle_events(self, "drag_started", response)
  handle_events(self, "dragged", response)
  handle_events(self, "drag_stopped", response)

  return response
end

local function render_from(vec, ui)
  vec:for_each(function(v)
    if v._render then
      if v.props.inactive then return end
      v:_render(ui)
    end
  end)
end

ui.Label = register_element("label", { text = "" }, function(self, ui)
  handle_reponse(self, ui:label(self.props.text, self.props.style))
end)

ui.Hyperlink = register_element("hyperlink", { text = "" }, function(self, ui)
  handle_reponse(self, ui:hyperlink(self.props.text, self.props.style))
end)

ui.Button = register_element("button", { text = "" }, function(self, ui)
  handle_reponse(self, ui:button(self.props.text, self.props.style))
end)

ui.Checkbox = register_element("checkbox", { text = "", checked = false }, function(self, ui)
  handle_reponse(self, handle_change(self, "checked", ui:checkbox(self.props.text, self.props.checked, self.props.style)))
end)

ui.Dragvalue = register_element("drag_value", { text = "", min = 0.0, max = 100.0, value = 0.0 }, function(self, ui)
  handle_reponse(self, handle_change(self, "value", ui:drag_value(self.props.text, self.props.value, self.props.style)))
end)

ui.Slider = register_element("slider", { text = "", value = 0.0 }, function(self, ui)
  handle_reponse(self, handle_change(self, "value", ui:drag_value(self.props.text, self.props.value, self.props.style)))
end)

ui.Separator = register_element("separator", {}, function(self, ui)
  handle_reponse(self, ui:separator(self.props.style))
end)

ui.Spinner = register_element("spinner", {}, function(self, ui)
  handle_reponse(self, ui:spinner(self.props.style))
end)

ui.ProgressBar = register_element("progress_bar", { value = 0.0, text = "" }, function(self, ui)
  handle_reponse(self, ui:progress_bar(self.props.value, self.props.text, self.props.style))
end)

ui.TextEditSingleLine = register_element("input", { value = "" }, function(self, ui)
  handle_reponse(self, handle_change(self, "value", ui:text_edit_singleline(self.props.value, self.props.style)))
end)
ui.TextEditMultiLine = register_element("textbox", { value = "" }, function(self, ui)
  handle_reponse(self, handle_change(self, "value", ui:text_edit_multiline(self.props.value, self.props.style)))
end)

ui.Align = register_element("align", { align = "start", layout = "left_to_right" }, function(self, ui)
  ui:align(self.props.layout, self.props.align, function(ui)
    render_from(self.children, ui)
  end, self.props.style)
end)

ui.CollapsingHeader = register_element("collapsing_header", { text = "" }, function(self, ui)
  ui:collapsing_header(self.props.text, function(ui)
    render_from(self.children, ui)
  end, self.props.style)
end)

ui.ScrollArea = register_element("scroll-area", {}, function(self, ui)
  ui:scroll_area(function(ui)
    render_from(self.children, ui)
  end, self.props.style)
end)

ui.HBox = register_element("hbox", {}, function(self, ui)
  ui:horizontal(function(ui)
    render_from(self.children, ui)
  end, self.props.style)
end)

ui.VBox = register_element("vbox", {}, function(self, ui)
  ui:vertical(function(ui)
    render_from(self.children, ui)
  end, self.props.style)
end)

ui.StyleWrapper = register_element("style", {}, function(self, ui)
  handle_style(self, ui)
  render_from(self.children, ui)
end)

ui.Frame = register_element("frame", { style = {} }, function(self, ui)
  ui:frame_block(self.props.style, function(ui)
    render_from(self.children, ui)
  end)
end)

ui.Scope = register_element("scope", { render = function(ui) end }, function(self, ui)
  ui:scope(function(ui)
    self.props.render(ui)
  end)
end)

ui.Handle = register_element("handle", { render = function(ui) end }, function(self, ui)
  self.props.render(ui)
end)

function lml_create(name, props, ...)
  if default_elements[name] then
    return default_elements[name](props, {...})
  end
end

local function render_ui(ui)
  render_from(elements, ui)
end

return function()
  register_font("DejaVuSansMono", DEJAVU_FONT_BYTES)

  return render_ui
end