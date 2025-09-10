local M = {}

local config = {
  log_path = vim.fn.expand '~' .. '/.edit-snooper.log',
  enabled = true,
}

function M.setup(user_config)
  config = vim.tbl_deep_extend('force', config, user_config or {})
end

local function log_change(change_type, offset, data)
  if not config.enabled then
    return
  end

  print(string.format('EditSnooper: Attempting to log %s at %d: %s', change_type, offset, data))
  print('EditSnooper: Log path: ' .. config.log_path)

  local file = io.open(config.log_path, 'a')
  if not file then
    print 'EditSnooper: Failed to open log file'
    return
  end

  local entry = string.format('%s,%d,%s\n', change_type, offset, data)
  file:write(entry)
  file:close()
end

-- Track insert mode changes (insertions and deletions)
local function on_text_changed_i()
  -- Tracks changes made in insert mode by comparing cursor position and line content
  -- between the current and previous state

  local bufnr = vim.api.nvim_get_current_buf()

  -- if last_insert doesn't exist then we can't calculate what changed, so just return
  if not M.last_insert then
    M.last_insert = {
      buf = bufnr,
      lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false),
      cursor = vim.api.nvim_win_get_cursor(0),
    }
    return
  end

  local current_lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local current_cursor = vim.api.nvim_win_get_cursor(0)
  local line = current_cursor[1] - 1

  -- Calculate the byte offset of the start of the current line
  -- by summing lengths of all previous lines (including newline chars)
  local offset = 0
  for i = 1, line do
    offset = offset + #M.last_insert.lines[i] + 1 -- +1 for newline
  end

  -- Detect insertions (cursor moved right)
  if current_cursor[2] > M.last_insert.cursor[2] then
    local col = M.last_insert.cursor[2]
    local inserted = string.sub(current_lines[line + 1] or '', col + 1, current_cursor[2])
    if inserted ~= '' then
      offset = offset + col -- Add column offset within line
      log_change('INSERT', offset, inserted)
    end

  -- Detect deletions (cursor moved left)
  elseif current_cursor[2] < M.last_insert.cursor[2] then
    local col = current_cursor[2]
    local old_line = M.last_insert.lines[line + 1] or ''
    local new_line = current_lines[line + 1] or ''
    local deleted = string.sub(old_line, col + 1, M.last_insert.cursor[2])
    if deleted ~= '' then
      offset = offset + col -- Add column offset within line
      log_change('DELETE', offset, tostring(#deleted))
    end
  end

  -- Update last_insert state for next comparison
  M.last_insert = {
    buf = bufnr,
    lines = current_lines,
    cursor = current_cursor,
  }
end

-- Track deletions from yank operations (normal mode deletes)
local function on_text_yank_post()
  -- Handles delete operations in normal mode by checking the unnamed register
  -- Only processes operations where the operator was 'd' (delete)

  if vim.v.event.operator ~= 'd' then
    return
  end

  local bufnr = vim.api.nvim_get_current_buf()
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    return
  end

  local cursor_pos = vim.api.nvim_win_get_cursor(0)
  local line = cursor_pos[1] - 1
  local col = cursor_pos[2]

  -- Calculate the byte offset of the cursor position by:
  -- 1. Summing lengths of all lines before current line (+1 for newlines)
  -- 2. Adding the column position in current line
  local offset = 0
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, line, false)
  for _, l in ipairs(lines) do
    offset = offset + #l + 1
  end
  offset = offset + col

  -- Get the actual deleted text from the unnamed register
  -- This properly handles multi-line and partial-line deletions
  local deleted_text = vim.fn.getreg '"'
  local length = #deleted_text
  log_change('DELETE', offset, tostring(length))
end

-- Track other text changes not caught by insert mode or yank handlers
local function on_text_changed()
  -- Maintains last_state for potential future diffing
  -- Currently just updates the state without processing changes

  local bufnr = vim.api.nvim_get_current_buf()

  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    return
  end

  if not M.last_state then
    M.last_state = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    return
  end

  -- Update last_state to current buffer content
  M.last_state = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
end

-- Initialize the plugin
function M.init()
  -- Set up autocommands
  vim.api.nvim_create_autocmd('TextChangedI', {
    callback = on_text_changed_i,
  })

  vim.api.nvim_create_autocmd('TextChanged', {
    callback = on_text_changed,
  })

  vim.api.nvim_create_autocmd('TextYankPost', {
    callback = on_text_yank_post,
  })

  -- Create user commands
  vim.api.nvim_create_user_command('EditSnooperStart', function()
    config.enabled = true
    vim.notify 'EditSnooper: Recording started'
  end, {})

  vim.api.nvim_create_user_command('EditSnooperStop', function()
    config.enabled = false
    vim.notify 'EditSnooper: Recording stopped'
  end, {})

  vim.api.nvim_create_user_command('EditSnooperToggle', function()
    config.enabled = not config.enabled
    vim.notify('EditSnooper: Recording ' .. (config.enabled and 'started' or 'stopped'))
  end, {})

  vim.api.nvim_create_user_command('EditSnooperSetPath', function(opts)
    config.log_path = opts.args
    vim.notify('EditSnooper: Log path set to ' .. opts.args)
  end, { nargs = 1 })
end

-- Auto-initialize when required
M.init()

return M
