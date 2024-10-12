INCLUDES = -Isrc/
WARNINGS = -Wall -Wuninitialized -Werror -Wno-unused-private-field
CXXFLAGS += -std=c++17 $(INCLUDES) $(WARNINGS)
CXX = @g++

# Find cpp files in subdirectories
SOURCES := $(shell find ./src -name '*.cpp')

# Find headers
HEADERS := $(shell find ./src -name '*.hpp' -o -name '*.h')

# Build directory
BUILD_DIR := _build

# Directory for object files
OBJDIR := $(BUILD_DIR)/obj

# Replace each source file with its corresponding object file
OBJECTS := $(patsubst ./%.cpp,$(OBJDIR)/%.o,$(SOURCES))

# Output files
BIN := $(BUILD_DIR)/bin
OUTPUT := $(BIN)/target

run: $(OUTPUT)
	@./$(OUTPUT)

make_directories:
	@mkdir -p $(OBJDIR)
	@mkdir -p $(BIN)

# The output depends on the object files and headers
$(OUTPUT): $(OBJECTS) make_directories
	$(CXX) $(CXXFLAGS) $(CPPFLAGS) -o $(OUTPUT) $(OBJECTS)

# Rule to generate object files
$(OBJDIR)/%.o: %.cpp $(HEADERS)
	@mkdir -p $(@D)
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -rdf $(BUILD_DIR)