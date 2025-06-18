#!/usr/bin/env ruby

# Read the safe version
content = File.read("Cargo.lock.safe")
packages = content.split("[[package]]").reject(&:empty?)
header = packages.shift

# Parse all packages with dependencies
all_packages = {}
packages.each do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  version_line = lines.find { |l| l.start_with?("version = ") }
  name = name_line.split('"')[1] if name_line
  version = version_line.split('"')[1] if version_line
  
  deps = []
  in_deps = false
  pkg.lines.each do |line|
    if line.strip == "dependencies = ["
      in_deps = true
    elsif line.strip == "]" && in_deps
      in_deps = false
    elsif in_deps && line.include?('"')
      dep_name = line.strip.split('"')[1]
      deps << dep_name if dep_name
    end
  end
  
  all_packages[name] = { 
    content: pkg, 
    version: version,
    deps: deps,
    depended_by: []
  }
end

# Calculate reverse dependencies
all_packages.each do |name, info|
  info[:deps].each do |dep|
    if all_packages[dep]
      all_packages[dep][:depended_by] << name
    end
  end
end

puts "Analyzing #{all_packages.length} packages"

# Find critical packages (many packages depend on them)
critical = all_packages.select { |name, info| info[:depended_by].length > 5 }
puts "\nCritical packages (>5 dependencies):"
critical.sort_by { |_, info| -info[:depended_by].length }.each do |name, info|
  puts "  #{name}: #{info[:depended_by].length} deps"
end

# Find leaf packages (no one depends on them)
leaves = all_packages.select { |name, info| info[:depended_by].empty? }
puts "\nLeaf packages: #{leaves.length}"

# Create minimal version with only critical path
# Start with workspace members
critical_path = ["backend", "graphql", "sql-entities", "actix-web", "async-graphql", "async-graphql-actix-web"]

# Add their direct dependencies
require 'set'
to_check = critical_path.dup
checked = Set.new
final_packages = Set.new

while !to_check.empty?
  pkg = to_check.shift
  next if checked.include?(pkg)
  checked.add(pkg)
  final_packages.add(pkg)
  
  if all_packages[pkg]
    # Add critical dependencies only
    all_packages[pkg][:deps].each do |dep|
      if all_packages[dep] && all_packages[dep][:depended_by].length > 2
        to_check << dep unless checked.include?(dep)
      end
    end
  end
end

# Always include actix-web-actors if present
if all_packages["actix-web-actors"]
  final_packages.add("actix-web-actors")
  all_packages["actix-web-actors"][:deps].each do |dep|
    final_packages.add(dep) if all_packages[dep]
  end
end

puts "\nCritical path packages: #{final_packages.size}"

# Create the minimal lockfile
minimal_packages = final_packages.select { |name| all_packages[name] }
minimal_content = header + minimal_packages.map { |name| "[[package]]" + all_packages[name][:content] }.join
File.write("Cargo.lock.critical", minimal_content)

puts "Created Cargo.lock.critical"

# Also print what notable packages are included
notable = ["actix", "actix-web", "actix-web-actors", "tokio", "futures-core", "serde", "bytes"]
puts "\nNotable packages included:"
notable.each do |pkg|
  if final_packages.include?(pkg)
    puts "  ✓ #{pkg} v#{all_packages[pkg][:version]}"
  else
    puts "  ✗ #{pkg}"
  end
end