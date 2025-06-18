#!/usr/bin/env ruby

# Read current Cargo.lock
content = File.read("Cargo.lock")
packages = content.split("[[package]]").reject(&:empty?)
header = packages.shift

# Parse and categorize packages
categories = {
  workspace: [],
  actix_core: [],
  async_graphql: [],
  tokio_runtime: [],
  serde_family: [],
  futures_family: [],
  proc_macro: [],
  utilities: [],
  other: []
}

packages.each do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  name = name_line.split('"')[1] if name_line
  
  case name
  when "backend", "graphql", "sql-entities"
    categories[:workspace] << name
  when /^actix/
    categories[:actix_core] << name
  when /^async-graphql/
    categories[:async_graphql] << name
  when /^tokio/, "mio", "parking_lot", "signal-hook-registry"
    categories[:tokio_runtime] << name
  when /^serde/, "itoa", "ryu"
    categories[:serde_family] << name
  when /^futures/
    categories[:futures_family] << name
  when "proc-macro2", "quote", "syn", /derive/, /-macros$/
    categories[:proc_macro] << name
  when "bytes", "pin-project-lite", "once_cell", "cfg-if", "log", "memchr", "libc"
    categories[:utilities] << name
  else
    categories[:other] << name
  end
end

puts "Package breakdown:"
categories.each do |cat, pkgs|
  puts "\n#{cat}: #{pkgs.length} packages"
  puts "  #{pkgs.sort.join(', ')}" if pkgs.any?
end

# Create a truly minimal version - remove everything we can
minimal_keep = [
  # Absolute minimum
  "backend", "graphql", "sql-entities",
  "actix-web", "async-graphql", "async-graphql-actix-web",
  # See what happens without these
  # "tokio", "futures-core", "serde", "bytes"
]

puts "\n\nCreating ultra-minimal version with only #{minimal_keep.length} packages..."

minimal_packages = packages.select do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  name = name_line.split('"')[1] if name_line
  minimal_keep.include?(name)
end

minimal_content = header + minimal_packages.join
File.write("Cargo.lock.ultra_minimal", minimal_content)
puts "Created Cargo.lock.ultra_minimal"