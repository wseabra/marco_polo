module Swimmable
  def swim
    puts "Swimming..."
  end
end

module Flyable
  def fly
    puts "Flying..."
  end
end

class Engine
  attr_reader :horsepower

  def initialize(horsepower)
    @horsepower = horsepower
  end

  def start
    puts "Engine started"
  end
end

class Car
  include Swimmable
  
  def initialize(engine, wheels)
    @engine = engine
    @wheels = wheels
  end

  def drive(destination)
    @engine.start
    puts "Driving to #{destination}"
  end
end

class AmphibiousVehicle < Car
  include Swimmable
  
  def mode_switch(surface_type)
    # switch mode
  end
end
