RONN_ENVIRONMENT = --manual="vimdir(1)"

all: vidir.1 vidir.1.html

vidir.1: vidir.1.ronn
	ronn $(RONN_ENVIRONMENT) --roff $<

vidir.1.html: vidir.1.ronn
	ronn $(RONN_ENVIRONMENT) --html $<

clean:
	rm -f vidir.1 vidir.1.html

.PHONY: all clean
